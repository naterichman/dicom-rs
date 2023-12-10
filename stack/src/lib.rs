use std::{collections::HashSet, path::PathBuf};

use dicom_object::{open_file, DefaultDicomObject, mem::{InMemDicomObject, InMemElement}};
use dicom_core::{value::{CastValueError, ConvertValueError}, Tag};
use lazy_static::lazy_static;
use dicom_dictionary_std::{tags, SOP_TO_CIOD, CIOD_TO_MODULES, MODULE_TO_ATTRIBUTES};
use snafu::prelude::*;

lazy_static!{
    pub static ref IGNORE_TAGS: Vec<Tag> = vec![
        tags::NUMBER_OF_FRAMES,
        tags::INSTANCE_NUMBER,
        tags::SOP_CLASS_UID,
        tags::SOP_INSTANCE_UID,
        tags::PIXEL_DATA,
        tags::SERIES_INSTANCE_UID,

        tags::IMAGE_ORIENTATION_PATIENT,
        tags::PIXEL_SPACING,
        tags::SLICE_THICKNESS,
        tags::SPACING_BETWEEN_SLICES,

        tags::IMAGE_TYPE,
        tags::ACQUISITION_DATE,
        tags::ACQUISITION_TIME,
        tags::IMAGE_POSITION_PATIENT,
        tags::WINDOW_CENTER,
        tags::WINDOW_WIDTH,
        tags::RESCALE_INTERCEPT,
        tags::RESCALE_SLOPE,
        tags::RESCALE_TYPE,
        tags::BODY_PART_EXAMINED,
    ];
}

#[derive(Debug, Snafu)]
pub enum StackError {
    #[snafu(display("Failed to read DICOM file {}: {}", path.display(), source))]
    Read{
        path: PathBuf,
        source: dicom_object::ReadError
    },
    #[snafu(display("Dicom at {}: no tag {}", path.display(), key))]
    MissingTag {
        key: String,
        path: PathBuf
    },
    CastTag{
        key: String,
        source: CastValueError
    },
    ConvertTag {
        key: String,
        source: ConvertValueError
    },
    IncosistentValues{
        key: String,
        values: Vec<String>
    },
    UnknownSOP{
        sop: String
    }
}
fn load_dicoms(files: Vec<PathBuf>) -> Result<Vec<DefaultDicomObject>, StackError> {
    let mut tfs = HashSet::new();
    let mut series = HashSet::new();
    let mut studies = HashSet::new();
    let mut modalities = HashSet::new();

    let mut datasets = Vec::new();
    for file in files {
        let ds = open_file(&file)
            .context(ReadSnafu {path: file.clone()})?;
        tfs.insert(ds.meta().transfer_syntax().to_string());
        series.insert(
            ds.get(tags::SERIES_INSTANCE_UID)
                .context(MissingTagSnafu { key: "SeriesInstanceUID".to_string(), path: file.clone()})?
                .to_str()
                .context(ConvertTagSnafu { key: "SeriesInstanceUID".to_string()})?
                .into_owned()
        );
        studies.insert(
            ds.get(tags::STUDY_INSTANCE_UID)
                .context(MissingTagSnafu { key: "StudyInstanceUId".to_string(), path: file.clone()})?
                .to_str()
                .context(ConvertTagSnafu { key: "StudyInstanceUID".to_string()})?
                .into_owned()
        );
        modalities.insert(
            ds.get(tags::MODALITY)
                .context(MissingTagSnafu { key: "Modality".to_string(), path: file.clone()})?
                .to_str()
                .context(ConvertTagSnafu { key: "Modality".to_string()})?
                .into_owned()
        );
        datasets.push(ds);
    }
    ensure!(tfs.len() == 1, IncosistentValuesSnafu{
        key: "TransferSyntaxUID".to_string(),
        values: tfs.into_iter().map(|v| v.to_string()).collect::<Vec<_>>()
    });
    ensure!(series.len() == 1, IncosistentValuesSnafu{
        key: "SeriesInstanceUID".to_string(),
        values: series.into_iter().collect::<Vec<String>>()
    });
    ensure!(studies.len() == 1, IncosistentValuesSnafu{
        key: "StudyInstanceUID".to_string(),
        values: studies.into_iter().collect::<Vec<_>>()
    });
    ensure!(modalities.len() == 1, IncosistentValuesSnafu{
        key: "Modality".to_string(),
        values: modalities.into_iter().collect::<Vec<_>>()
    });
    Ok(datasets)
}

pub fn process_dicoms(files: Vec<PathBuf>) -> Result<(), StackError> {
    println!("Here");
    let datasets = load_dicoms(files.clone())?;
    println!("Here1");
    let sop = datasets[0].get(tags::SOP_CLASS_UID)
        .context(MissingTagSnafu { key: "SOPClassUID".to_string(), path: files[0].clone()})?
        .to_str()
        .context(ConvertTagSnafu { key: "SOPClassUID".to_string()})?;

    let ciod = *SOP_TO_CIOD.get(&sop)
        .context(UnknownSOPSnafu{sop: sop.into_owned()})?;
    let modules = *CIOD_TO_MODULES.get(ciod).unwrap();
    let attrs = modules.iter().map(|module| {
            *MODULE_TO_ATTRIBUTES.get(module.module_id).unwrap()
        })
        .flatten()
        .collect::<Vec<_>>();

    let dataset = InMemoryDicomObject::new();
    println!("{:?}", modules);
    Ok(())
}