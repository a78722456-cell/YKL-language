use std::collections::HashMap as StdHashMap;
use std::hash::Hash;

pub mod collections {
    use super::*;

    pub struct Vector<T> {
        inner: Vec<T>,
    }

    impl<T> Vector<T> {
        pub fn new() -> Self {
            Self { inner: Vec::new() }
        }
        pub fn push(&mut self, item: T) {
            self.inner.push(item);
        }
        pub fn pop(&mut self) -> Option<T> {
            self.inner.pop()
        }
        pub fn len(&self) -> usize {
            self.inner.len()
        }
        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }
        pub fn get(&self, index: usize) -> Option<&T> {
            self.inner.get(index)
        }
    }

    pub struct HashMap<K, V> {
        inner: StdHashMap<K, V>,
    }

    impl<K: Eq + Hash, V> HashMap<K, V> {
        pub fn new() -> Self {
            Self {
                inner: StdHashMap::new(),
            }
        }
        pub fn insert(&mut self, k: K, v: V) -> Option<V> {
            self.inner.insert(k, v)
        }
        pub fn get(&self, k: &K) -> Option<&V> {
            self.inner.get(k)
        }
    }

    pub struct RingBuffer<T> {
        buffer: Vec<Option<T>>,
        head: usize,
        tail: usize,
        capacity: usize,
    }

    impl<T: Clone> RingBuffer<T> {
        pub fn new(capacity: usize) -> Self {
            Self {
                buffer: vec![None; capacity],
                head: 0,
                tail: 0,
                capacity,
            }
        }
        pub fn push(&mut self, item: T) -> bool {
            let next = (self.head + 1) % self.capacity;
            if next == self.tail {
                return false; // full
            }
            self.buffer[self.head] = Some(item);
            self.head = next;
            true
        }
        pub fn pop(&mut self) -> Option<T> {
            if self.tail == self.head {
                return None; // empty
            }
            let item = self.buffer[self.tail].take();
            self.tail = (self.tail + 1) % self.capacity;
            item
        }
    }
}

pub mod io {
    use std::fs;
    use std::io::{self, Write};

    pub fn print(s: &str) {
        print!("{}", s);
        let _ = io::stdout().flush();
    }

    pub fn println(s: &str) {
        println!("{}", s);
    }

    pub fn read_file(path: &str) -> Result<String, io::Error> {
        fs::read_to_string(path)
    }

    pub fn write_file(path: &str, content: &str) -> Result<(), io::Error> {
        fs::write(path, content)
    }
}

pub mod net {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    pub struct HttpServer {
        port: u16,
    }

    impl HttpServer {
        pub fn bind(port: u16) -> Self {
            Self { port }
        }

        pub fn handle_one<F>(&self, handler: F) -> Result<(), std::io::Error>
        where
            F: Fn(&str) -> String,
        {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))?;
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buf) {
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let resp_body = handler(&req);
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                        resp_body.len(),
                        resp_body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                }
            }
            Ok(())
        }
    }
}

pub mod serialization {
    pub fn to_json_string(key: &str, val: &str) -> String {
        format!("{{\"{}\": \"{}\"}}", key, val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collections() {
        let mut vec = collections::Vector::new();
        vec.push(10);
        assert_eq!(vec.pop(), Some(10));

        let mut rb = collections::RingBuffer::new(3);
        assert!(rb.push("a"));
        assert!(rb.push("b"));
        assert_eq!(rb.pop(), Some("a"));
    }
}
