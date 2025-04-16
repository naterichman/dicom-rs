use dicom_ul::association::client::ClientAssociationOptions;
mod common;

static SCU_AE_TITLE: &str = "ECHO-SCU";
static SCP_AE_TITLE: &str = "ECHO-SCP";

static IMPLICIT_VR_LE: &str = "1.2.840.10008.1.2";
static EXPLICIT_VR_LE: &str = "1.2.840.10008.1.2.1";
static JPEG_BASELINE: &str = "1.2.840.10008.1.2.4.50";
static VERIFICATION_SOP_CLASS: &str = "1.2.840.10008.1.1";
static DIGITAL_MG_STORAGE_SOP_CLASS: &str = "1.2.840.10008.5.1.4.1.1.1.2";

/// Run an SCP and an SCU concurrently, negotiate an association and release it.
#[test]
fn scu_scp_association_test() {
    use common::spawn_scp;
    let (scp_handle, scp_addr) = spawn_scp(vec![VERIFICATION_SOP_CLASS.to_string()]).unwrap();

    let association = ClientAssociationOptions::new()
        .calling_ae_title(SCU_AE_TITLE)
        .called_ae_title(SCP_AE_TITLE)
        .with_presentation_context(VERIFICATION_SOP_CLASS, vec![IMPLICIT_VR_LE, EXPLICIT_VR_LE])
        .with_presentation_context(
            DIGITAL_MG_STORAGE_SOP_CLASS,
            vec![IMPLICIT_VR_LE, EXPLICIT_VR_LE, JPEG_BASELINE],
        )
        .establish(scp_addr)
        .unwrap();

    association
        .release()
        .expect("did not have a peaceful release");

    scp_handle
        .join()
        .expect("SCP panicked")
        .expect("Error at the SCP");
}

#[cfg(feature = "async")]
#[tokio::test(flavor = "multi_thread")]
async fn scu_scp_asociation_test() {
    use common::spawn_scp_async;
    let (scp_handle, scp_addr) = spawn_scp_async().await.unwrap();

    let association = ClientAssociationOptions::new()
        .calling_ae_title(SCU_AE_TITLE)
        .called_ae_title(SCP_AE_TITLE)
        .with_presentation_context(VERIFICATION_SOP_CLASS, vec![IMPLICIT_VR_LE, EXPLICIT_VR_LE])
        .with_presentation_context(
            DIGITAL_MG_STORAGE_SOP_CLASS,
            vec![IMPLICIT_VR_LE, EXPLICIT_VR_LE, JPEG_BASELINE],
        )
        .establish_async(scp_addr)
        .await
        .unwrap();

    association
        .release()
        .await
        .expect("did not have a peaceful release");

    scp_handle
        .await
        .expect("SCP panicked")
        .expect("Error at the SCP");
}
