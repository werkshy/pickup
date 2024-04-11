use rand::{distributions::Alphanumeric, thread_rng, Rng};

const ID_LEN: usize = 16;

pub fn generate_id() -> String {
    generate_id_len(ID_LEN)
}

fn generate_id_len(len: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{generate_id, generate_id_len};

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_eq!(16, id1.len());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_len() {
        let id = generate_id_len(20);
        assert_eq!(20, id.len());
    }
}
