
use crate::*;
use apl_hir::{HirExpr, HirExprKind, HirFunction, HirBlock, HirStmt};

pub struct MirBuilder {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    current_block: BasicBlockId,
    loop_scopes: Vec<(BasicBlockId, BasicBlockId)>,
    scopes: Vec<Vec<LocalId>>,
}

impl MirBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            locals: Vec::new(),
            blocks: Vec::new(),
            current_block: BasicBlockId(0),
            loop_scopes: Vec::new(),
            scopes: Vec::new(),
        };
        // start block
        builder.blocks.push(BasicBlock {
            statements: Vec::new(),
            terminator: Terminator::Unreachable, // placeholder
        });
        builder
    }

    pub fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock {
            statements: Vec::new(),
            terminator: Terminator::Unreachable,
        });
        id
    }

    pub fn add_local(&mut self, id: LocalId, ty: Type, span: Span, mutability: bool, is_param: bool) {
        self.locals.push(LocalDecl { id, ty, span, mutability, is_param });
    }

    pub fn build(mut self, func: &HirFunction) -> MirBody {
        self.add_local(LocalId(0), func.return_type.clone(), func.span, true, false);
        // Register params
        for param in &func.params {
            self.add_local(param.id, param.ty.clone(), param.span, false, true);
        }

        if let Some(body) = &func.body {
            self.lower_block(body);
        }
        self.blocks[self.current_block.0 as usize].terminator = Terminator::Return(func.span);

        MirBody {
            locals: self.locals,
            blocks: self.blocks,
            start_block: BasicBlockId(0),
        }
    }

    fn lower_block(&mut self, block: &HirBlock) {
        self.scopes.push(Vec::new());
        for stmt in &block.stmts {
            self.lower_stmt(stmt);
        }
        let dropped_locals = self.scopes.pop().unwrap();
        for local_id in dropped_locals.into_iter().rev() {
            // Find the local decl to check if it's Copy
            let is_copy = self.locals.iter().find(|l| l.id == local_id).map(|l| l.ty.is_copy()).unwrap_or(true);
            if !is_copy {
                let span = block.span; // fallback span
                self.blocks[self.current_block.0 as usize].statements.push(Statement::Drop(Place::Local(local_id), span));
            }
            // Always inject StorageDead
            let span = block.span;
            self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageDead(local_id, span));
        }
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { id, name: _, mutability, ty, init, span } => {
                self.add_local(*id, ty.clone(), *span, *mutability, false);
                self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageLive(*id, *span));
                if let Some(expr) = init {
                    let rvalue = self.lower_expr(expr);
                    self.blocks[self.current_block.0 as usize].statements.push(
                        Statement::Assign(Place::Local(*id), rvalue, *span)
                    );
                }
            }
            HirStmt::Expr(expr) => {
                self.lower_expr(expr);
            }
            HirStmt::Return(opt_expr, span) => {
                if let Some(expr) = opt_expr {
                    // Usually return assigns to a special _0 local, but for now we just lower it.
                    let rval = self.lower_expr(expr);
                    self.blocks[self.current_block.0 as usize].statements.push(
                        Statement::Assign(Place::Local(LocalId(0)), rval, *span)
                    ); 
                }
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Return(*span);
                // Create a new block for dead code after return
                self.current_block = self.new_block();
            }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Rvalue {
        match &expr.kind {
            HirExprKind::LiteralInt(v) => Rvalue::Use(Operand::Constant(Constant::Int(*v, expr.ty.clone()))),
            HirExprKind::LiteralFloat(v) => Rvalue::Use(Operand::Constant(Constant::Float(*v, expr.ty.clone()))),
            HirExprKind::LiteralBool(v) => Rvalue::Use(Operand::Constant(Constant::Bool(*v))),
            HirExprKind::LiteralString(v) => Rvalue::Use(Operand::Constant(Constant::String(v.clone()))),
                        HirExprKind::LocalRef(id) => {
                if expr.ty.is_copy() {
                    Rvalue::Use(Operand::Copy(Place::Local(*id)))
                } else {
                    Rvalue::Use(Operand::Move(Place::Local(*id)))
                }
            }
            HirExprKind::BinaryOp(op, l, r) => {
                let l_rv = self.lower_expr(l);
                let r_rv = self.lower_expr(r);
                let temp_l = self.make_temp(l_rv, l.span, l.ty.clone());
                let temp_r = self.make_temp(r_rv, r.span, r.ty.clone());
                Rvalue::BinaryOp(op.clone(), Operand::Copy(temp_l), Operand::Copy(temp_r))
            }
                        HirExprKind::UnaryOp(op, inner) => {
                match op {
                    apl_ast::UnaryOp::Ref => {
                        let place = self.lower_place(inner);
                        Rvalue::Ref(crate::Region(0), BorrowKind::Shared, place)
                    }
                    apl_ast::UnaryOp::RefMut => {
                        let place = self.lower_place(inner);
                        Rvalue::Ref(crate::Region(0), BorrowKind::Mut, place)
                    }
                    _ => {
                        let inner_rv = self.lower_expr(inner);
                        let temp = self.make_temp(inner_rv, inner.span, inner.ty.clone());
                        Rvalue::UnaryOp(op.clone(), Operand::Copy(temp))
                    }
                }
            }
            HirExprKind::Assign(place_expr, rvalue_expr) => {
                // To do this correctly, we need to evaluate the place_expr as a Place, not Rvalue
                // For now, assume it's just LocalRef or FieldAccess
                let dest_place = self.lower_place(place_expr);
                let rvalue = self.lower_expr(rvalue_expr);
                self.blocks[self.current_block.0 as usize].statements.push(Statement::Assign(dest_place, rvalue, expr.span));
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Call { callee, args } => {
                let mut op_args = Vec::new();
                for arg in args {
                    let arg_rv = self.lower_expr(arg);
                    let temp = self.make_temp(arg_rv, arg.span, arg.ty.clone());
                    op_args.push(Operand::Copy(temp));
                }
                let dest = self.make_temp_uninit(expr.ty.clone(), expr.span);
                let next_block = self.new_block();
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Call {
                    function: *callee,
                    args: op_args,
                    destination: dest.clone(),
                    target: next_block,
                    span: expr.span,
                };
                self.current_block = next_block;
                Rvalue::Use(Operand::Copy(dest))
            }
            HirExprKind::If { cond, then_branch, else_branch } => {
                let cond_rv = self.lower_expr(cond);
                let cond_place = self.make_temp(cond_rv, cond.span, Type::Bool);
                
                let then_block = self.new_block();
                let else_block = self.new_block();
                let merge_block = self.new_block();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::SwitchInt {
                    discriminant: Operand::Copy(cond_place),
                    targets: vec![(1, then_block)],
                    otherwise: else_block,
                };
                
                let dest = self.make_temp_uninit(expr.ty.clone(), expr.span);
                
                // Then branch
                self.current_block = then_block;
                let then_rv = self.lower_expr(then_branch);
                self.blocks[self.current_block.0 as usize].statements.push(Statement::Assign(dest.clone(), then_rv, then_branch.span));
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(merge_block);
                
                // Else branch
                self.current_block = else_block;
                if let Some(eb) = else_branch {
                    let else_rv = self.lower_expr(eb);
                    self.blocks[self.current_block.0 as usize].statements.push(Statement::Assign(dest.clone(), else_rv, eb.span));
                }
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(merge_block);
                
                self.current_block = merge_block;
                Rvalue::Use(Operand::Copy(dest))
            }

            HirExprKind::Loop(body) => {
                let loop_block = self.new_block();
                let exit_block = self.new_block();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(loop_block);
                self.current_block = loop_block;
                
                self.loop_scopes.push((loop_block, exit_block));
                self.lower_expr(body);
                self.loop_scopes.pop();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(loop_block);
                self.current_block = exit_block;
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Break => {
                if let Some(&(_, break_bb)) = self.loop_scopes.last() {
                    self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(break_bb);
                    self.current_block = self.new_block(); // dead code
                }
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Continue => {
                if let Some(&(continue_bb, _)) = self.loop_scopes.last() {
                    self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(continue_bb);
                    self.current_block = self.new_block(); // dead code
                }
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::While { cond, body } => {
                let cond_block = self.new_block();
                let body_block = self.new_block();
                let exit_block = self.new_block();
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(cond_block);
                self.current_block = cond_block;
                
                let cond_rv = self.lower_expr(cond);
                let cond_place = self.make_temp(cond_rv, cond.span, Type::Bool);
                
                self.blocks[self.current_block.0 as usize].terminator = Terminator::SwitchInt {
                    discriminant: Operand::Copy(cond_place),
                    targets: vec![(1, body_block)],
                    otherwise: exit_block,
                };
                
                self.current_block = body_block;
                self.loop_scopes.push((cond_block, exit_block));
                self.lower_expr(body);
                self.loop_scopes.pop();
                self.blocks[self.current_block.0 as usize].terminator = Terminator::Goto(cond_block);
                
                self.current_block = exit_block;
                Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
            }
            HirExprKind::Block(b, opt_e) => {
                self.lower_block(b);
                if let Some(e) = opt_e {
                    self.lower_expr(e)
                } else {
                    Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void)))
                }
            }
            HirExprKind::FieldAccess(base, _name) => {
                let base_place = self.lower_place(base); // needs proper lower_place logic
                // we need FieldId, assume 0 for now as dummy
                let dest = Place::Field { base: Box::new(base_place), field: FieldId(0) };
                Rvalue::Use(Operand::Copy(dest))
            }
                        HirExprKind::StructInit { struct_id: _, fields } => {
                let mut ops = Vec::new();
                for (_, field_expr) in fields {
                    let f_rv = self.lower_expr(field_expr);
                    let temp = self.make_temp(f_rv, field_expr.span, field_expr.ty.clone());
                    ops.push(Operand::Copy(temp));
                }
                Rvalue::Aggregate(ops)
            }
            _ => Rvalue::Use(Operand::Constant(Constant::Int(0, Type::Void))),
        }
    }

    fn lower_place(&mut self, expr: &HirExpr) -> Place {
        match &expr.kind {
            HirExprKind::LocalRef(id) => Place::Local(*id),
            HirExprKind::FieldAccess(base, _) => {
                let base_place = self.lower_place(base);
                Place::Field { base: Box::new(base_place), field: FieldId(0) } // dummy 0
            }
            _ => {
                // If it's a complex expression, make a temp
                let rvalue = self.lower_expr(expr);
                self.make_temp(rvalue, expr.span, expr.ty.clone())
            }
        }
    }


    fn make_temp_uninit(&mut self, ty: Type, span: Span) -> Place {
        let id = LocalId(1000 + self.locals.len() as u32);
        self.add_local(id, ty, span, true, false);
        self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageLive(id, span));
        Place::Local(id)
    }

    fn make_temp(&mut self, rvalue: Rvalue, span: Span, ty: Type) -> Place {

        // Allocate a high ID for temp local (hack for now until we have a proper ID allocator)
        let id = LocalId(1000 + self.locals.len() as u32);
        self.add_local(id, ty, span, true, false);
        
        self.blocks[self.current_block.0 as usize].statements.push(Statement::StorageLive(id, span));
        let place = Place::Local(id);
        self.blocks[self.current_block.0 as usize].statements.push(Statement::Assign(place.clone(), rvalue, span));
        place
    }
}

