use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Identity {
    raw: Arc<()>,
}

impl Identity {
    pub fn new() -> Self {
        Identity { raw: Arc::new(()) }
    }
}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.raw, &other.raw)
    }
}

impl Eq for Identity {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn not_equals() {
        let n = 0xFFFFF;
        let mut ids = Vec::new();

        for _ in 0..n {
            let id = Identity::new();
            assert!(ids.iter().all(|e| e != &id));
            ids.push(id);
        }
    }
}
