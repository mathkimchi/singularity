use singularity_macros::{Datable, Packet, PacketUnion};
use singularity_sap::{
    datable::{ToData, TryFromData},
    packet::{IdType, PacketTrait},
};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplayUnits},
    ui_element::UIElement,
};

/// Not how I would actually do paste, but I just want to test with multiple unknown size fields.
#[derive(Datable, Packet, PartialEq, Debug)]
struct PastePacket {
    clean_content: String,
    f: f64,
    raw_content: String,
}

#[derive(Datable, Packet, PartialEq, Debug)]
struct A {
    d: DisplayArea,
}
#[derive(Datable, Packet, PartialEq, Debug)]
struct ResizeEvent(DisplayArea);
#[derive(Datable, Packet, PartialEq, Eq, Debug)]
struct FocusedEvent;

#[derive(Datable, Packet, PartialEq, Debug)]
enum Enum {
    Unit(()),
    F(f32),
    FocusedEvent(FocusedEvent),
    P(PastePacket),
    Tuple((FocusedEvent, PastePacket)),
    Recursive(Box<Enum>),
}

#[derive(PacketUnion, Packet, PartialEq, Debug)]
enum MyPacket {
    P(PastePacket),
    A(A),
    R(ResizeEvent),
    F(FocusedEvent),
    E(Enum),
}

#[test]
fn test_data_conversion() {
    let integer = -32986584;
    assert_eq!(i32::try_from_data(&integer.to_data()), Some(integer));

    let float = -83712465.2;
    assert_eq!(f64::try_from_data(&float.to_data()), Some(float));

    let string = "Hello!".to_string();
    assert_eq!(String::try_from_data(&string.to_data()), Some(string));

    let tuple = ();
    // dbg!(&tuple.to_data());
    assert_eq!(
        <() as TryFromData>::try_from_data(&tuple.to_data()),
        Some(tuple)
    );

    let tuple = (2,);
    // dbg!(&tuple.to_data());
    assert_eq!(
        <(i32,) as TryFromData>::try_from_data(&tuple.to_data()),
        Some(tuple)
    );

    let tuple = (2, 3.);
    assert_eq!(
        <(i32, f64) as TryFromData>::try_from_data(&tuple.to_data()),
        Some(tuple)
    );

    let tuple = (2, 0.3, "Hi!".to_string());
    assert_eq!(
        <(i32, f64, String) as TryFromData>::try_from_data(&tuple.to_data()),
        Some(tuple)
    );

    let p = PastePacket {
        clean_content: "SAFMNDSBUUasdlbvajk sdjkflasdhfuioaef".to_string(),
        f: 42.0,
        raw_content: "pi3hfajksld jkla;dsu292\nasdlkfj".to_string(),
    };
    assert_eq!(PastePacket::try_from_data(&p.to_data()), Some(p));

    let d = DisplayUnits::from_mixed(2, 3.0);
    assert_eq!(DisplayUnits::try_from_data(&d.to_data()), Some(d));

    let c = DisplayCoord::new(DisplayUnits::from_mixed(2, 3.0), 2.into());
    assert_eq!(DisplayCoord::try_from_data(&c.to_data()), Some(c));

    let a = A {
        d: DisplayArea(
            DisplayCoord::new(0.into(), (-2.276).into()),
            DisplayCoord::new(3.1.into(), 86352.into()),
        ),
    };
    assert_eq!(A::try_from_data(&a.to_data()), Some(a));

    let p = MyPacket::E(Enum::Recursive(Box::new(Enum::F(3.))));
    assert_eq!(MyPacket::try_from_data(&p.to_data()), Some(p));

    let v = vec![
        MyPacket::E(Enum::Recursive(Box::new(Enum::F(3.)))),
        MyPacket::F(FocusedEvent),
    ];
    assert_eq!(<Vec<MyPacket>>::try_from_data(&v.to_data()), Some(v));

    let a = [
        MyPacket::E(Enum::Recursive(Box::new(Enum::F(3.)))),
        MyPacket::F(FocusedEvent),
    ];
    assert_eq!(<[MyPacket; 2]>::try_from_data(&a.to_data()), Some(a));

    let ui_element = UIElement::Backgrounded(
        Box::new(UIElement::Text("Sup?".to_string())),
        Color::LIGHT_GREEN,
    );
    assert_eq!(
        UIElement::try_from_data(&ui_element.to_data()),
        Some(ui_element)
    );
}
