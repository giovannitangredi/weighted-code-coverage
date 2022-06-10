use std::fs;
use std::path::*;

use csv;
use serde_json::json;

use crate::error::Error;
use crate::Metrics;

// Export metrics on a csv in the specified path
pub(crate) fn export_to_csv(
    csv_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    project_coverage: f64,
) -> Result<(), Error> {
    let mut writer = csv::Writer::from_path(csv_path)?;
    writer.write_record(&[
        "FILE",
        "SIFIS PLAIN",
        "SIFIS QUANTIZED",
        "CRAP",
        "SKUNK",
        "IGNORED",
        "IS COMPLEX",
        "FILE PATH",
    ])?;
    metrics.iter().try_for_each(|m| -> Result<(), Error> {
        writer.write_record(&[
            &m.file,
            &format!("{:.3}", m.sifis_plain),
            &format!("{:.3}", m.sifis_quantized),
            &format!("{:.3}", m.crap),
            &format!("{:.3}", m.skunk),
            &format!("{}", false),
            &format!("{}", m.is_complex),
            &m.file_path,
        ])?;
        Ok(())
    })?;
    writer.write_record(&[
        "PROJECT_COVERAGE",
        format!("{:.3}", project_coverage).as_str(),
        "-",
        "-",
        "-",
        "-",
        "-",
        "-",
    ])?;
    writer.write_record(&[
        "LIST OF COMPLEX FILES",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
    ])?;
    complex_files
        .iter()
        .try_for_each(|m| -> Result<(), Error> {
            writer.write_record(&[
                &m.file,
                &format!("{:.3}", m.sifis_plain),
                &format!("{:.3}", m.sifis_quantized),
                &format!("{:.3}", m.crap),
                &format!("{:.3}", m.skunk),
                &format!("{}", false),
                &format!("{}", m.is_complex),
                &m.file_path,
            ])?;
            Ok(())
        })?;
    writer.write_record(&[
        "TOTAL COMPLEX FILES",
        format!("{:?}", complex_files.len()).as_str(),
        "",
        "",
        "",
        "",
        "",
        "",
    ])?;
    writer.write_record(&[
        "LIST OF IGNORED FILES",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
        "----------",
    ])?;
    files_ignored
        .iter()
        .try_for_each(|file| -> Result<(), Error> {
            writer.write_record(&[
                file.as_str(),
                format!("{:.3}", 0.).as_str(),
                format!("{:.3}", 0.).as_str(),
                format!("{:.3}", 0.).as_str(),
                format!("{:.3}", 0.).as_str(),
                format!("{}", true).as_str(),
                "-",
                "-",
            ])?;
            Ok(())
        })?;
    writer.write_record(&[
        "TOTAL FILES IGNORED",
        format!("{:?}", files_ignored.len()).as_str(),
        "",
        "",
        "",
        "",
        "",
        "",
    ])?;
    writer.flush()?;
    Ok(())
}

// Export all metrics to a json file
pub(crate) fn export_to_json(
    project_folder: &Path,
    output_path: &Path,
    metrics: Vec<Metrics>,
    files_ignored: Vec<String>,
    complex_files: Vec<Metrics>,
    project_coverage: f64,
) -> Result<(), Error> {
    let n_files = files_ignored.len();
    let number_of_complex_files = complex_files.len();
    let json = json!({
        "project": project_folder.display().to_string(),
        "number_of_files_ignored": n_files,
        "number_of_complex_files": number_of_complex_files,
        "metrics":metrics,
        "files_ignored":files_ignored,
        "complex_files": complex_files,
        "project_coverage" : project_coverage,
    });
    let json_string = serde_json::to_string(&json)?;
    fs::write(output_path, json_string)?;
    Ok(())
}
