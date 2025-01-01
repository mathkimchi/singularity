pub type IdType = u64;

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: std::marker::Sized {
    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: &[u8]) -> Option<Self>;
    // fn from_data(data: &[u8]) -> Self;
}

pub trait EventTrait: PacketTrait {
    const EVENT_TYPE_ID: IdType;
}

/// NOTE: The subevents are actually both idents and types.
/// Idents can be types, but types can't be idents (easily),
/// which is why I told the macro subevents are idents.
#[macro_export]
macro_rules! combine_events {
    // ($($v:vis)? $new_name:ident => [$($subevent:ty),*]) => {
    //     enum $new_name {}
    // };

    // I guess vis is special, so no need for the optional with ?
    ($v:vis $new_name:ident => [$($subevent:ident),*]) => {
        $v enum $new_name {
            $($subevent($subevent),)*
        }

        impl $crate::sap::packet::PacketTrait for $new_name {
            fn from_data(data: &[u8]) -> Option<Self> {
                let (id, data) = seperate_id(data);
                match id {
                    $($subevent::EVENT_TYPE_ID => Some(Self::$subevent($subevent::from_data(data)?)),)*
                    _ => None,
                }
            }

            fn to_data(&self) -> Vec<u8> {
                let (id, data) = match self {
                    $(Self::$subevent(subevent) => ($subevent::EVENT_TYPE_ID, subevent.to_data()),)*
                };

                add_id(id, &data)
            }
        }
    };
}