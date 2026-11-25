use chrono::Datelike;
use http_body_util::Full;
use hyper::{body::Bytes, service::service_fn, Response};
use hyper_util::rt::TokioIo;
use look_out_api::{checker::Executor, config::Config};
use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair};
use std::sync::Arc;
use tokio_rustls::{
    rustls::{self, pki_types::PrivatePkcs8KeyDer},
    TlsAcceptor,
};

#[tokio::test]
async fn trusted_near_expiry_certificate_is_flagged_and_untrusted_certificate_is_rejected() {
    let mut ca_params = CertificateParams::new(vec![]).expect("CA params");
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_key = KeyPair::generate().expect("CA key");
    let ca_cert = ca_params.self_signed(&ca_key).expect("CA");
    let issuer = Issuer::new(ca_params, ca_key);
    let mut params =
        CertificateParams::new(vec!["localhost".into(), "127.0.0.1".into()]).expect("params");
    let after = chrono::Utc::now() + chrono::Duration::days(3);
    params.not_after = rcgen::date_time_ymd(after.year(), after.month() as u8, after.day() as u8);
    let key = KeyPair::generate().expect("key");
    let cert = params.signed_by(&key, &issuer).expect("cert");
    let tls = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![cert.der().clone(), ca_cert.der().clone()],
            PrivatePkcs8KeyDer::from(key.serialize_der()).into(),
        )
        .expect("tls");
    let acceptor = TlsAcceptor::from(Arc::new(tls));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listen");
    let address = listener.local_addr().expect("address");
    let handle = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(stream) = acceptor.accept(stream).await else {
                    return;
                };
                let service = service_fn(|_| async {
                    Ok::<_, std::convert::Infallible>(Response::new(Full::new(Bytes::from_static(
                        b"ok",
                    ))))
                });
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), service)
                    .await;
            });
        }
    });
    let dir = tempfile::tempdir().expect("directory");
    let ca_path = dir.path().join("ca.pem");
    std::fs::write(&ca_path, ca_cert.pem()).expect("CA file");
    let base = format!("[[target]]\nname='tls'\ntype='security'\nurl='https://{address}'\n");
    let config =
        Config::parse(&format!("{base}ca_cert_path='{}'", ca_path.display())).expect("config");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(
        result
            .findings
            .iter()
            .any(|finding| finding.rule == "tls_cert_expiry" && finding.severity == "high"),
        "{result:?}"
    );
    let config = Config::parse(&base).expect("config");
    let result = Executor::new(config.targets[0].clone(), config.defaults, None)
        .expect("executor")
        .check()
        .await;
    assert!(!result.ok);
    assert!(result
        .error
        .as_deref()
        .expect("reason")
        .contains("TLS validation"));
    handle.abort();
}
