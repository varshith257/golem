use std::path::Path;
use wasm_metadata::Metadata;
use wasmparser::{ComponentExternalKind, Parser, Payload, Validator, WasmFeatures};

fn read_bytes(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn main() {
    let bytes = read_bytes(Path::new(
        "/home/vigoo/projects/zivergetech/golem/integration-tests/src/it/wasm/shopping-cart.wasm",
    ))
    .unwrap();
    let metadata = Metadata::from_binary(&bytes).unwrap();

    println!("metadata: {:?}", metadata);

    let mut depth = 0;
    let mut found_table_manipulation = false;

    let mut features = WasmFeatures::default();
    features.component_model = true;
    features.component_model_values = true;
    features.simd = true;

    let mut validator = Validator::new_with_features(features);
    //let mut current_module_sections: Option<Sections<'_, CoreIndexSpace>> = None;

    let parser = Parser::new(0);
    let module = wasm_ast::core::Module::try_from((parser, bytes.as_slice())).unwrap();
    println!("module: {:?}", module);

    let parser = Parser::new(0);
    for payload in parser.parse_all(&bytes) {
        match payload {
            Ok(payload) => {
                //println!("payload: {:?}", payload);
                validator.payload(&payload).unwrap();

                // match &mut current_module_sections {
                //     None => {}
                //     Some(sections) => {
                //         sections.add(&payload);
                //     }
                // }

                match payload {
                    Payload::Version { .. } => {}
                    Payload::TypeSection(_) => {}
                    Payload::ImportSection(_) => {}
                    Payload::FunctionSection(_) => {}
                    Payload::TableSection(_) => {}
                    Payload::MemorySection(_) => {}
                    Payload::TagSection(_) => {}
                    Payload::GlobalSection(_) => {}
                    Payload::ExportSection(_) => {}
                    Payload::StartSection { .. } => {}
                    Payload::ElementSection(_) => {}
                    Payload::DataCountSection { .. } => {}
                    Payload::DataSection(_) => {}
                    Payload::CodeSectionStart { .. } => {}
                    Payload::CodeSectionEntry(body) => {
                        for op in body.get_operators_reader().unwrap() {
                            match op.unwrap() {
                                wasmparser::Operator::TableSet { .. } => {
                                    found_table_manipulation = true;
                                }
                                wasmparser::Operator::TableFill { .. } => {
                                    found_table_manipulation = true;
                                }
                                wasmparser::Operator::TableCopy { .. } => {
                                    found_table_manipulation = true;
                                }
                                wasmparser::Operator::TableInit { .. } => {
                                    found_table_manipulation = true;
                                }
                                _ => {}
                            }
                        }
                    }
                    Payload::ModuleSection { .. } => {
                        depth += 1;
                        //current_module_sections = Some(Sections::new_core());
                    }
                    Payload::InstanceSection(_) => {}
                    Payload::CoreTypeSection(_) => {}
                    Payload::ComponentSection { .. } => {
                        depth += 1;
                    }
                    Payload::ComponentInstanceSection(_) => {}
                    Payload::ComponentAliasSection(_) => {}
                    Payload::ComponentTypeSection(_) => {}
                    Payload::ComponentCanonicalSection(_) => {}
                    Payload::ComponentStartSection { .. } => {}
                    Payload::ComponentImportSection(_) => {}
                    Payload::ComponentExportSection(export_reader) => {
                        if depth == 0 {
                            for exp in export_reader {
                                let exp = exp.unwrap();
                                println!("Export {:?}", exp);

                                match exp.kind {
                                    ComponentExternalKind::Module => {}
                                    ComponentExternalKind::Func => {}
                                    ComponentExternalKind::Value => {}
                                    ComponentExternalKind::Type => {}
                                    ComponentExternalKind::Instance => {
                                        let index = exp.index;
                                        let types = validator.types(0).unwrap();
                                        let component_instance = types.component_instance_at(index);
                                        println!("Component instance {:?}", component_instance);
                                    }
                                    ComponentExternalKind::Component => {}
                                }
                            }
                        }
                    }
                    Payload::CustomSection(_) => {}
                    Payload::UnknownSection { .. } => {}
                    Payload::End(_) => {
                        // match current_module_sections.take() {
                        //     Some(sections) => {
                        //         let module = Module::from_sections(sections);
                        //         println!("Module {:?}", module);
                        //     }
                        //     None => {}
                        // }
                        depth -= 1;
                    }
                }
            }
            Err(err) => {
                println!("error: {:?}", err);
            }
        }
    }

    println!("Found table manipulation: {found_table_manipulation}");
}
