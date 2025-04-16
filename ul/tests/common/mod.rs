
use dicom_ul::pdu::{Pdu, PresentationContextResult, PresentationContextResultReason};
use std::{borrow::Cow, net::SocketAddr};
use dicom_ul::association::server::{ServerAssociationOptions, ServerAssociation};
    use tokio::net::TcpStream;
// static SCU_AE_TITLE: &str = "ECHO-SCU";
static SCP_AE_TITLE: &str = "ECHO-SCP";

static IMPLICIT_VR_LE: &str = "1.2.840.10008.1.2";
// static EXPLICIT_VR_LE: &str = "1.2.840.10008.1.2.1";
// static JPEG_BASELINE: &str = "1.2.840.10008.1.2.4.50";
// static VERIFICATION_SOP_CLASS: &str = "1.2.840.10008.1.1";
// static DIGITAL_MG_STORAGE_SOP_CLASS: &str = "1.2.840.10008.5.1.4.1.1.1.2";
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;


pub fn spawn_scp(syntaxes: Vec<String>) -> Result<(std::thread::JoinHandle<Result<()>>, SocketAddr)> {
    let listener = std::net::TcpListener::bind("localhost:0")?;
    let addr = listener.local_addr()?;
    let mut scp = ServerAssociationOptions::new()
        .accept_called_ae_title()
        .ae_title(SCP_AE_TITLE);
    for syntax in syntaxes.iter() {
        scp = scp.with_transfer_syntax(syntax.clone());
    }

    let h = std::thread::spawn(move || -> Result<()> {
        let (stream, _addr) = listener.accept()?;
        let mut association = scp.establish(stream)?;

        assert_eq!(
            association.presentation_contexts(),
            &[
                PresentationContextResult {
                    id: 1,
                    reason: PresentationContextResultReason::Acceptance,
                    transfer_syntax: IMPLICIT_VR_LE.to_string(),
                },
                PresentationContextResult {
                    id: 3,
                    reason: PresentationContextResultReason::AbstractSyntaxNotSupported,
                    transfer_syntax: IMPLICIT_VR_LE.to_string(),
                }
            ],
        );

        // handle one release request
        let pdu = association.receive()?;
        assert_eq!(pdu, Pdu::ReleaseRQ);
        association.send(&Pdu::ReleaseRP)?;

        Ok(())
    });
    Ok((h, addr))
}
