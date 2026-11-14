use apl_ast::Program;
use apl_borrowck::BorrowChecker;
use apl_codegen::{CCodeGenerator, CodegenFunction};
use apl_hir::lowering::HirLowering;
use apl_lexer::Lexer;
use apl_mir::interpret::{Interpreter, Value};
use apl_mir::lowering::MirBuilder;
use apl_parser::Parser;
use apl_sema::SemanticAnalyzer;
use std::collections::HashMap;

fn run_ykl_in_vm(source: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(source, 0);
    let tokens = lexer.tokenize().map_err(|e| format!("Lex: {:?}", e))?;

    let mut parser = Parser::new(tokens);
    let ast: Program = parser.parse_program();
    if let Some(err) = parser.errors.first() {
        return Err(format!("Parse: {}", err));
    }

    let mut resolved_items = Vec::new();
    for item in ast.items {
        if let apl_ast::Item::Import(rel_path, _) = item {
            let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let ws_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap_or(manifest_dir);
            let candidates = [
                std::path::Path::new(&rel_path).to_path_buf(),
                ws_root.join(&rel_path),
                ws_root.join("stdlib").join(&rel_path),
            ];
            if let Some(target_path) = candidates.into_iter().find(|c| c.exists()) {
                let sub_src = std::fs::read_to_string(&target_path).map_err(|e| format!("{}", e))?;
                let mut sub_lexer = Lexer::new(&sub_src, 0);
                let sub_tokens = sub_lexer.tokenize().map_err(|e| format!("{:?}", e))?;
                let mut sub_parser = Parser::new(sub_tokens);
                let sub_prog = sub_parser.parse_program();
                for sub_item in sub_prog.items {
                    if !matches!(sub_item, apl_ast::Item::Import(_, _)) {
                        resolved_items.push(sub_item);
                    }
                }
            }
        } else {
            resolved_items.push(item);
        }
    }
    let ast = Program { items: resolved_items };

    let mut sema = SemanticAnalyzer::new();
    sema.analyze_program(&ast)
        .map_err(|errs| format!("Sema: {:?}", errs))?;

    let mut hir_lower = HirLowering::new();
    let hir = hir_lower
        .lower_program(&ast)
        .map_err(|errs| format!("HIR: {:?}", errs))?;

    let mut interp = Interpreter::new();
    let mut bc = BorrowChecker::new();

    for func in &hir.functions {
        let mut mir_body = MirBuilder::new_with_structs(&hir.structs).build(func);
        apl_opt::MirOptimizer::optimize_body(&mut mir_body);
        bc.check_body(&mir_body)
            .map_err(|errs| format!("BorrowCk: {:?}", errs))?;
        interp.register_function(func.id, &func.name, mir_body);
    }

    interp
        .call_by_name("main", vec![])
        .map_err(|e| format!("VM: {:?}", e))
}

fn compile_ykl_to_c(source: &str, bare_metal: bool) -> String {
    let mut lexer = Lexer::new(source, 0);
    let tokens = lexer.tokenize().unwrap();
    let ast = Parser::new(tokens).parse_program();
    let mut sema = SemanticAnalyzer::new();
    sema.analyze_program(&ast).unwrap();
    let hir = HirLowering::new().lower_program(&ast).unwrap();

    let mut funcs = Vec::new();
    for f in &hir.functions {
        funcs.push(CodegenFunction {
            id: f.id,
            name: f.name.clone(),
            return_type: f.return_type.clone(),
            body: MirBuilder::new_with_structs(&hir.structs).build(f),
        });
    }

    let mut structs = HashMap::new();
    let mut enums = HashMap::new();
    for item in &ast.items {
        match item {
            apl_ast::Item::Struct(s) => {
                structs.insert(s.name.clone(), s.clone());
            }
            apl_ast::Item::Enum(e) => {
                enums.insert(e.name.clone(), e.clone());
            }
            _ => {}
        }
    }

    CCodeGenerator::generate_program(&funcs, &structs, &enums, &[], bare_metal)
}

#[test]
fn test_ykl2_ergonomics_and_lower_place() {
    let src = r#"
        struct Motor {
            rpm: int
            active: byte

            fn set_speed(target: int) {
                self.rpm = target
                self.active = 1
            }
        }

        fn main() -> int {
            m := Motor { rpm: 0, active: 0 }
            m.set_speed(3200)
            return m.rpm
        }
    "#;
    let res = run_ykl_in_vm(src).expect("Ошибка запуска YKL 2.0");
    assert_eq!(res, Value::Int(3200));
}

#[test]
fn test_hex_binary_and_modbus_crc16() {
    let src = r#"
        fn main() -> int {
            crc := 0xFFFF
            data := 0b0000_0001
            crc = crc ^ data
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001
            } else {
                crc = crc >> 1
            }
            return crc
        }
    "#;
    let res = run_ykl_in_vm(src).expect("Ошибка в побитовой логике Modbus CRC16");
    // 0xFFFF ^ 1 = 0xFFFE; (0xFFFE & 1) == 0 -> 0xFFFE >> 1 = 0x7FFF (32767)
    assert_eq!(res, Value::Int(0x7FFF));
}

#[test]
fn test_mmio_emulation_and_bare_metal_codegen() {
    let src = r#"
        fn main() -> int {
            mmio_write(0x4002_1000, 0x0000_0001)
            bit_set(0x4002_1000, 4)
            return mmio_read(0x4002_1000)
        }
    "#;
    let res = run_ykl_in_vm(src).expect("Ошибка эмуляции MMIO");
    assert_eq!(res, Value::Int(0x0000_0011));

    let c_code = compile_ykl_to_c(src, true);
    assert!(c_code.contains("volatile uint32_t*"));
    assert!(!c_code.contains("#include <stdio.h>"));
}

#[test]
fn test_mentor_rejects_manual_lifetime() {
    let src = r#"
        fn bad<'a>(x: &'a int) -> int {
            return 0
        }
    "#;
    let mut lexer = Lexer::new(src, 0);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            let msg = format!("{:?}", e);
            assert!(msg.contains("запрещён в YKL"));
            return;
        }
    };
    let mut parser = Parser::new(tokens);
    let _ = parser.parse_program();
    assert!(!parser.errors.is_empty(), "Expected error for manual lifetime");
    let msg = format!("{}", parser.errors[0]);
    assert!(msg.contains("запрещён в YKL"));
}

#[test]
fn test_enum_match_and_packed_struct() {
    let src = r#"
        @packed
        struct CanPacket {
            id: u16
            dlc: u8
            data: [u8; 8]
        }

        enum CncState {
            Idle,
            Homing,
            Cutting(i32),
            EmergencyStop(u8),
        }

        fn process_cnc(state: CncState) -> i32 {
            match state {
                CncState::Idle => 0,
                CncState::Homing => 10,
                CncState::Cutting(rpm) => rpm,
                CncState::EmergencyStop(code) => {
                    mmio_write(0x4002_0C14, 0)
                    return 0 - code
                }
            }
        }

        fn main() -> int {
            state := CncState::Cutting(4500)
            return process_cnc(state)
        }
    "#;
    let res = run_ykl_in_vm(src).expect("Ошибка исполнения enum/match");
    assert_eq!(res, Value::Int(4500));

    let c_code = compile_ykl_to_c(src, true);
    assert!(c_code.contains("__attribute__((packed))"));
    assert!(c_code.contains("enum CncState_Tag"));
    assert!(c_code.contains("struct CncState"));
}


#[test]
fn test_multifile_import_and_driver() {
    let dir = std::env::temp_dir().join("ykl_test_multifile");
    let _ = std::fs::remove_dir_all(&dir);
    let drivers_dir = dir.join("drivers");
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&drivers_dir).unwrap();
    std::fs::create_dir_all(&src_dir).unwrap();

    let gpio_yk = r#"
        struct GpioPin {
            port_odr_addr: int
            pin_number: int

            fn high() {
                bit_set(self.port_odr_addr, self.pin_number)
            }
        }
    "#;
    std::fs::write(drivers_dir.join("stm32_gpio.yk"), gpio_yk).unwrap();

    let main_yk = r#"
        import "../drivers/stm32_gpio.yk"

        fn main() -> int {
            pin := GpioPin {
                port_odr_addr: 0x4002_0C14,
                pin_number: 12
            }
            pin.high()
            return pin.pin_number
        }
    "#;
    let main_path = src_dir.join("main.yk");
    std::fs::write(&main_path, main_yk).unwrap();

    let mut parser = Parser::new(Lexer::new(main_yk, 0).tokenize().unwrap());
    let prog = parser.parse_program();
    assert!(prog.items.iter().any(|item| matches!(item, apl_ast::Item::Import(path, _) if path.contains("stm32_gpio.yk"))));

    let _ = std::fs::remove_dir_all(&dir);
}


#[test]
fn test_stdlib_hal_drivers() {
    let src = r#"
        import "stdlib/hal/gpio.yk"
        import "stdlib/hal/pwm.yk"
        import "stdlib/hal/modbus_can.yk"

        fn main() -> int {
            pin := DigitalPin {
                odr_addr: 0x4002_0C14,
                idr_addr: 0x4002_0C10,
                pin: 12
            }
            pin.set_high()

            pwm := PwmChannel {
                ccr_addr: 0x4001_0034,
                min_us: 1000,
                max_us: 2000
            }
            pwm.set_throttle_percent(50)

            crc := modbus_crc16_step(0xFFFF, 0x01)
            return crc
        }
    "#;
    let res = run_ykl_in_vm(src).expect("Ошибка запуска HAL");
    assert_ne!(res, Value::Int(0));
}
