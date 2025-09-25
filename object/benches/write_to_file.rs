use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dicom_object::mem::InMemDicomObject;
use dicom_object::meta::FileMetaTableBuilder;
use dicom_core::{VR, Tag, DataElement, dicom_value, PrimitiveValue};
use tempfile::tempdir;

fn create_test_object() -> InMemDicomObject {
    let mut obj = InMemDicomObject::new_empty();

    // Add some typical DICOM data elements
    obj.put(DataElement::new(
        Tag(0x0010, 0x0010),
        VR::PN,
        dicom_value!(Strs, ["Doe^John"]),
    ));
    obj.put(DataElement::new(
        Tag(0x0008, 0x0060),
        VR::CS,
        dicom_value!(Strs, ["CR"]),
    ));
    obj.put(DataElement::new(
        Tag(0x0008, 0x0018),
        VR::UI,
        dicom_value!(Strs, ["1.4.645.212121"]),
    ));
    obj.put(DataElement::new(
        Tag(0x0020, 0x000d),
        VR::UI,
        dicom_value!(Strs, ["1.2.840.10008.1.2.1"]),
    ));
    obj.put(DataElement::new(
        Tag(0x0008, 0x0016),
        VR::UI,
        dicom_value!(Strs, ["1.2.840.10008.5.1.4.1.1.1"]),
    ));

    obj
}

fn create_large_test_object() -> InMemDicomObject {
    let mut obj = create_test_object();

    // Add pixel data to simulate a real medical image
    let pixel_data = vec![0u8; 512 * 512 * 2]; // 512x512 16-bit image
    obj.put(DataElement::new(
        Tag(0x7fe0, 0x0010),
        VR::OW,
        PrimitiveValue::from(pixel_data),
    ));

    // Add image dimensions
    obj.put(DataElement::new(
        Tag(0x0028, 0x0010),
        VR::US,
        dicom_value!(U16, [512]),
    ));
    obj.put(DataElement::new(
        Tag(0x0028, 0x0011),
        VR::US,
        dicom_value!(U16, [512]),
    ));
    obj.put(DataElement::new(
        Tag(0x0028, 0x0100),
        VR::US,
        dicom_value!(U16, [16]),
    ));

    obj
}

fn bench_write_small_file(c: &mut Criterion) {
    let obj = create_test_object();
    let file_obj = obj.with_meta(
        FileMetaTableBuilder::default()
            .transfer_syntax("1.2.840.10008.1.2.1")
            .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.1")
            .media_storage_sop_instance_uid("1.4.645.212121")
    ).unwrap();

    c.bench_function("write_small_dicom_file", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test.dcm");
            black_box(file_obj.write_to_file(&file_path).unwrap());
        });
    });
}

fn bench_write_large_file(c: &mut Criterion) {
    let obj = create_large_test_object();
    let file_obj = obj.with_meta(
        FileMetaTableBuilder::default()
            .transfer_syntax("1.2.840.10008.1.2.1")
            .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.1")
            .media_storage_sop_instance_uid("1.4.645.212121")
    ).unwrap();

    c.bench_function("write_large_dicom_file", |b| {
        b.iter(|| {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("test_large.dcm");
            black_box(file_obj.write_to_file(&file_path).unwrap());
        });
    });
}

fn bench_write_multiple_small_files(c: &mut Criterion) {
    let obj = create_test_object();

    c.bench_function("write_multiple_small_files", |b| {
        b.iter(|| {
            for i in 0..5 {
                let file_obj = obj.clone().with_meta(
                    FileMetaTableBuilder::default()
                        .transfer_syntax("1.2.840.10008.1.2.1")
                        .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.1")
                        .media_storage_sop_instance_uid(&format!("1.4.645.212121.{}", i))
                ).unwrap();

                let dir = tempdir().unwrap();
                let file_path = dir.path().join(format!("test_{}.dcm", i));
                black_box(file_obj.write_to_file(&file_path).unwrap());
            }
        });
    });
}

fn bench_write_different_transfer_syntaxes(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_different_transfer_syntaxes");

    let obj = create_test_object();

    let transfer_syntaxes = vec![
        ("explicit_vr_little_endian", "1.2.840.10008.1.2.1"),
        ("implicit_vr_little_endian", "1.2.840.10008.1.2"),
        ("explicit_vr_big_endian", "1.2.840.10008.1.2.2"),
    ];

    for (name, ts_uid) in transfer_syntaxes {
        let file_obj = obj.clone().with_meta(
            FileMetaTableBuilder::default()
                .transfer_syntax(ts_uid)
                .media_storage_sop_class_uid("1.2.840.10008.5.1.4.1.1.1")
                .media_storage_sop_instance_uid("1.4.645.212121")
        ).unwrap();

        group.bench_function(name, |b| {
            b.iter(|| {
                let dir = tempdir().unwrap();
                let file_path = dir.path().join("test.dcm");
                black_box(file_obj.write_to_file(&file_path).unwrap());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_write_small_file,
    bench_write_large_file,
    bench_write_multiple_small_files,
    bench_write_different_transfer_syntaxes
);
criterion_main!(benches);