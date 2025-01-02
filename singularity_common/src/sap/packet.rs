pub type IdType = u64;

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: std::marker::Sized {
    const PACKET_TYPE_ID: IdType;

    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: &[u8]) -> Option<Self>;
    // fn from_data(data: &[u8]) -> Self;
}

/// returns the id (from the beginning) and the rest of the data
pub fn split_id(data: &[u8]) -> (IdType, &[u8]) {
    let (id_bytes, inner_data) = data.split_at((IdType::BITS / 8) as usize);

    let id = IdType::from_be_bytes(id_bytes.try_into().unwrap());

    (id, inner_data)
}
pub fn join_id(id: IdType, inner_data: &[u8]) -> Vec<u8> {
    let id_bytes: &[u8] = &id.to_be_bytes();

    [id_bytes, inner_data].concat()
}

/// NOTE: The subevents are actually both idents and types.
/// Idents can be types, but types can't be idents (easily),
/// which is why I told the macro subevents are idents.
#[macro_export]
macro_rules! packet_union {
    // ($($v:vis)? $new_name:ident => [$($subevent:ty),*]) => {
    //     enum $new_name {}
    // };

    // I guess vis is special, so no need for the optional with ?
    ($v:vis $new_name:ident => [$($subevent:ident),*], $event_id:expr) => {
        $v enum $new_name {
            $($subevent($subevent),)*
        }

        impl $crate::sap::packet::PacketTrait for $new_name {
            const PACKET_TYPE_ID: IdType = $event_id;

            fn from_data(data: &[u8]) -> Option<Self> {
                let (id, data) = $crate::sap::packet::split_id(data);
                match id {
                    $($subevent::PACKET_TYPE_ID => Some(Self::$subevent($subevent::from_data(data)?)),)*
                    _ => None,
                }
            }

            fn to_data(&self) -> Vec<u8> {
                let (id, data) = match self {
                    $(Self::$subevent(subevent) => ($subevent::PACKET_TYPE_ID, subevent.to_data()),)*
                };

                $crate::sap::packet::join_id(id, &data)
            }
        }
    };
}
