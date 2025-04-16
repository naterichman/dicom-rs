use dicom_dictionary_std::uids::VERIFICATION;
use dicom_ul::ClientAssociationOptions;
use rstest::rstest;
use std::{net::TcpStream, time::Instant};

mod common;

const TIMEOUT_TOLERANCE: u64 = 25;

#[rstest]
#[case(100)]
#[case(500)]
#[case(1000)]
fn test_slow_association(#[case] timeout: u64) {
    let scu_init = ClientAssociationOptions::new()
        .with_abstract_syntax(VERIFICATION)
        .calling_ae_title("RANDOM")
        .read_timeout(std::time::Duration::from_secs(1))
        .connection_timeout(std::time::Duration::from_millis(timeout));

    let now = Instant::now();
    let _res = scu_init.establish_with("RANDOM@167.167.167.167:11111");
    let elapsed = now.elapsed();
    assert!(
        elapsed.as_millis() < (timeout + TIMEOUT_TOLERANCE).into(),
        "Elapsed time {}ms exceeded the timeout {}ms",
        elapsed.as_millis(),
        timeout
    );
}

#[cfg(feature = "async")]
#[rstest]
#[case(100)]
#[case(500)]
#[case(1000)]
#[tokio::test(flavor = "multi_thread")]
async fn test_slow_association_async(#[case] timeout: u64) {
    let scu_init = ClientAssociationOptions::new()
        .with_abstract_syntax(VERIFICATION)
        .calling_ae_title("RANDOM")
        .read_timeout(std::time::Duration::from_secs(1))
        .connection_timeout(std::time::Duration::from_millis(timeout));
    let now = Instant::now();
    let res = scu_init
        .establish_with_async("RANDOM@167.167.167.167:11111")
        .await;
    assert!(res.is_err());
    let elapsed = now.elapsed();
    println!("Elapsed time: {:?}", elapsed);
    assert!(
        elapsed.as_millis() < (timeout + TIMEOUT_TOLERANCE).into(),
        "Elapsed time {}ms exceeded the timeout {}ms",
        elapsed.as_millis(),
        timeout
    );
}

#[cfg(feature = "async")]
pub async fn establish_async() -> (Vec<u8>, TcpStream, SocketAddr) {
    use std::io::Cursor;

    use bytes::{Buf, BytesMut};
    use dicom_ul::{pdu::{AssociationAC, AssociationRQ, UserVariableItem, MAXIMUM_PDU_SIZE}, read_pdu, write_pdu, IMPLEMENTATION_CLASS_UID, IMPLEMENTATION_VERSION_NAME};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = std::net::TcpListener::bind("localhost:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let max_pdu_length = MAXIMUM_PDU_SIZE;
    let mut read_buffer = BytesMut::with_capacity(MAXIMUM_PDU_SIZE as usize);

    let pdu = loop {
        let mut buf = Cursor::new(&read_buffer[..]);
        match read_pdu(&mut buf, MAXIMUM_PDU_SIZE, false).unwrap()
        {
            Some(pdu) => {
                read_buffer.advance(buf.position() as usize);
                break pdu;
            }
            None => {
                // Reset position
                buf.set_position(0)
            }
        }
        let recv = socket
            .read_buf(&mut read_buffer)
            .await.unwrap();
        assert!(recv > 0);
    };

    let mut buffer: Vec<u8> = Vec::with_capacity(max_pdu_length as usize);
    match pdu {
        Pdu::AssociationRQ(AssociationRQ {
            calling_ae_title,
            called_ae_title,
            application_context_name,
            presentation_contexts,
            ..
        }) => {
            // treat 0 as the maximum size admitted by the standard

            let presentation_contexts: Vec<_> = presentation_contexts
                .into_iter()
                .map(|pc| {
                    let (transfer_syntax, reason) =
                            (
                                "1.2.840.10008.1.2".to_string(),
                                PresentationContextResultReason::Acceptance,
                            );

                    PresentationContextResult {
                        id: pc.id,
                        reason,
                        transfer_syntax,
                    }
                })
                .collect();

            write_pdu(
                &mut buffer,
                &Pdu::AssociationAC(AssociationAC {
                    protocol_version: 1,
                    application_context_name,
                    presentation_contexts: presentation_contexts.clone(),
                    calling_ae_title: calling_ae_title.clone(),
                    called_ae_title,
                    user_variables: vec![
                        UserVariableItem::MaxLength(max_pdu_length),
                        UserVariableItem::ImplementationClassUID(
                            IMPLEMENTATION_CLASS_UID.to_string(),
                        ),
                        UserVariableItem::ImplementationVersionName(
                            IMPLEMENTATION_VERSION_NAME.to_string(),
                        ),
                    ],
                }),
            )
            .unwrap();
            socket.write_all(&buffer).await.unwrap();

            (buffer, socket, addr)
        }
        _ => panic!("Expected AssociationRQ, got {:?}", pdu),
    }
}

#[cfg(feature = "async")]
#[test]
fn test_quick_successive_pdus(){
    let (buffer, socket, addr) = establish_async();
    let scu= ClientAssociationOptions::new()
        .with_abstract_syntax(VERIFICATION)
        .calling_ae_title("RANDOM")
        .read_timeout(std::time::Duration::from_secs(1))
        .connection_timeout(std::time::Duration::from_millis(timeout))
        .establish(addr);
}