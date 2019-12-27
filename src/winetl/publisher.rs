use serde_json::Value;
use winapi::um::winevt::*;
use crate::winevt::EvtHandle;
use crate::errors::WinThingError;
use crate::winevt::wevtapi::evt_get_publisher_metadata_property;
use crate::winevt::wevtapi::evt_open_publisher_metadata;
use crate::winevt::wevtapi::evt_open_publisher_enum;
use crate::winevt::wevtapi::evt_next_publisher_id;


const PUBLISHER_META_REFERENCES: [(&str, u32); 5] = [
    ("EvtPublisherMetadataChannelReferencePath", EvtPublisherMetadataChannelReferencePath),
    ("EvtPublisherMetadataChannelReferenceIndex", EvtPublisherMetadataChannelReferenceIndex),
    ("EvtPublisherMetadataChannelReferenceID", EvtPublisherMetadataChannelReferenceID),
    ("EvtPublisherMetadataChannelReferenceFlags", EvtPublisherMetadataChannelReferenceFlags),
    ("EvtPublisherMetadataChannelReferenceMessageID", EvtPublisherMetadataChannelReferenceMessageID)
];

const PUBLISHER_META_TASKS: [(&str, u32); 4] = [
    ("EvtPublisherMetadataTaskName", EvtPublisherMetadataTaskName),
    ("EvtPublisherMetadataTaskEventGuid", EvtPublisherMetadataTaskEventGuid),
    ("EvtPublisherMetadataTaskValue", EvtPublisherMetadataTaskValue),
    ("EvtPublisherMetadataTaskMessageID", EvtPublisherMetadataTaskMessageID)
];

const PUBLISHER_META_LEVELS: [(&str, u32); 3] = [
    ("EvtPublisherMetadataLevelName", EvtPublisherMetadataLevelName),
    ("EvtPublisherMetadataLevelValue", EvtPublisherMetadataLevelValue),
    ("EvtPublisherMetadataLevelMessageID", EvtPublisherMetadataLevelMessageID)
];

const PUBLISHER_META_OPCODES: [(&str, u32); 3] = [
    ("EvtPublisherMetadataOpcodeName", EvtPublisherMetadataOpcodeName),
    ("EvtPublisherMetadataOpcodeValue", EvtPublisherMetadataOpcodeValue),
    ("EvtPublisherMetadataOpcodeMessageID", EvtPublisherMetadataOpcodeMessageID)
];

const PUBLISHER_META_KEYWORDS: [(&str, u32); 3] = [
    ("EvtPublisherMetadataKeywordName", EvtPublisherMetadataKeywordName),
    ("EvtPublisherMetadataKeywordValue", EvtPublisherMetadataKeywordValue),
    ("EvtPublisherMetadataKeywordMessageID", EvtPublisherMetadataKeywordMessageID)
];

const PUBLISHER_METADATA: [(&str, u32); 12] = [
    ("EvtPublisherMetadataPublisherGuid", EvtPublisherMetadataPublisherGuid),
    ("EvtPublisherMetadataResourceFilePath", EvtPublisherMetadataResourceFilePath),
    ("EvtPublisherMetadataParameterFilePath", EvtPublisherMetadataParameterFilePath),
    ("EvtPublisherMetadataMessageFilePath", EvtPublisherMetadataMessageFilePath),
    ("EvtPublisherMetadataHelpLink", EvtPublisherMetadataHelpLink),
    ("EvtPublisherMetadataPublisherMessageID", EvtPublisherMetadataPublisherMessageID),
    ("EvtPublisherMetadataChannelReferences", EvtPublisherMetadataChannelReferences),
    ("EvtPublisherMetadataLevels", EvtPublisherMetadataLevels),
    ("EvtPublisherMetadataTasks", EvtPublisherMetadataTasks),
    ("EvtPublisherMetadataOpcodes", EvtPublisherMetadataOpcodes),
    ("EvtPublisherMetadataKeywords", EvtPublisherMetadataKeywords),
    ("EvtPublisherMetadataPropertyIdEND", EvtPublisherMetadataPropertyIdEND)
];


#[derive(Debug)]
pub struct PublisherMeta {
    pub name: String,
    handle: EvtHandle
}

impl PublisherMeta {
    pub fn new(name: String) -> Result<Self, WinThingError> {
        let handle = evt_open_publisher_metadata(
            &None,
            Some(name.clone()),
            None
        )?;

        Ok(
            Self {
                name: name,
                handle: handle
            }
        )
    }

    pub fn to_json_value(&self) -> Result<Value, WinThingError> {
        let mut mapping = json!({});

        for (key, id) in &PUBLISHER_METADATA {
            match id {
                &EvtPublisherMetadataPropertyIdEND => {
                    break;
                },
                _ => {
                    let variant = match evt_get_publisher_metadata_property(
                        &self.handle, *id
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("Error getting metadata property {}: {:?}", key, e);
                            continue;
                        }
                    };

                    match variant.get_json_value() {
                        Ok(v) => {
                            mapping[key] = v;
                        },
                        Err(e) => {
                            error!("Error getting variant value: {:?}", e);
                        }
                    }
                }
            }
        }

        Ok(mapping)
    }
}


#[derive(Debug)]
pub struct PublisherEnumerator {
    session: Option<EvtHandle>,
    pub_enum_handle: EvtHandle
}

impl PublisherEnumerator {
    pub fn new(session: Option<EvtHandle>) -> Result<Self, WinThingError> {
        let handle = evt_open_publisher_enum(
            &session
        )?;

        Ok(
            Self {
                session: session,
                pub_enum_handle: handle
            }
        )
    }
}
impl Iterator for PublisherEnumerator {
    type Item = PublisherMeta;

    fn next(&mut self) -> Option<PublisherMeta> {
        loop {
            match evt_next_publisher_id(
                &self.pub_enum_handle
            ){
                Ok(o) => {
                    match o {
                        Some(name) => {
                            let meta = match PublisherMeta::new(name.clone()) {
                                Ok(m) => m,
                                Err(e) => {
                                    error!("Error openting meta for {}: {:?}", name, e);
                                    continue;
                                }
                            };

                            return Some(meta)
                        },
                        None => break
                    };
                },
                Err(e) => {
                    error!("Error on evt_next_publisher_id for PublisherEnumerator: {:?}", e);
                    continue;
                }
            }
        }

        None
    }
}


// #[derive(Debug)]
// pub struct IterPublisher {
//     pub_enum: PublisherEnumerator
// }

// impl IterPublisher {
//     pub fn new(pub_enum: PublisherEnumerator) -> Self {
//         IterPublisher {
//             pub_enum: pub_enum
//         }
//     }
// }