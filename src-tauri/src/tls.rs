// expose talks tls to the local league and riot clients and to the public
// data dragon and github endpoints. ring is the one crypto provider in the
// build, installed for the whole process before anything opens a connection:
// reqwest is built without a provider of its own and uses this one.

pub fn install_crypto() {
    // a second call finds the provider already in place, which is fine.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clients_build_with_ring() {
        // safe to call more than once: the second install finds one in place.
        install_crypto();
        install_crypto();
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
        // reqwest has no provider of its own, so this only works with ring
        // installed as the process default.
        assert!(reqwest::Client::builder().build().is_ok());
    }
}
