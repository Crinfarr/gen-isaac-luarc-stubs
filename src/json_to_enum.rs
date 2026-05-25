use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result},
    path::Path,
};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TEnumVal {
    TString(String),
    TInt(i32),
    TObjS { value: String, comment: String },
    TObjI { value: usize, comment: String },
    TMap(HashMap<String, TEnumVal>),
}

pub type EnumFile = HashMap<String, HashMap<String, TEnumVal>>;
pub fn decode_enum_file(filepath: &Path) -> Result<EnumFile> {
    let file_bytes = {
        let mut fb = vec![];
        File::open(filepath)?.read_to_end(&mut fb)?;
        fb
    };
    Ok(serde_json::from_slice::<EnumFile>(&file_bytes)?)
}
pub fn to_lua(decoded_file: &EnumFile) -> Result<String> {
    let mut fstring = String::default();
    for (enum_name, enum_data) in decoded_file.iter() {
        fstring += &format!("--@enum {enum_name}\n{enum_name} = {{\n");
        for (variant_name, tval) in enum_data {
            match tval {
                TEnumVal::TString(enum_val) => {
                    fstring += &format!("\t{variant_name} = {enum_val},\n")
                }
                TEnumVal::TInt(enum_val) => fstring += &format!("\t{variant_name} = {enum_val},\n"),
                TEnumVal::TObjS { value, comment } => {
                    fstring += &format!("\t--{comment}\n\t{variant_name} = {value},\n")
                }
                TEnumVal::TObjI { value, comment } => {
                    fstring += &format!("\t--{comment}\n\t{variant_name} = {value},\n")
                }
                TEnumVal::TMap(m) => {
                    fstring += &format!("\t--@enum {variant_name}\n\t{variant_name} = {{\n");
                    for (subname, subval) in m {
                        match subval {
                            TEnumVal::TInt(subval) => {
                                fstring += &format!("\t\t{subname} = {subval},\n")
                            }
                            TEnumVal::TString(subval) => {
                                fstring += &format!("\t\t{subname} = {subval},\n")
                            }
                            TEnumVal::TObjI { value, comment } => {
                                fstring += &format!("\t\t--{comment}\n\t\t{subname} = {value},\n")
                            }
                            TEnumVal::TObjS { value, comment } => {
                                fstring += &format!("\t\t--{comment}\n\t\t{subname} = {value},\n")
                            }
                            TEnumVal::TMap(_) => {
                                panic!(
                                    "filloax if you ever make this error happen i will kill us both"
                                )
                            }
                        }
                    }
                    fstring += "\t},\n"
                }
            }
        }
        fstring += "}\n"
    }
    Ok(fstring)
}
