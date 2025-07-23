//! Illustrates how "reflection" works in Bevy.
//!
//! Reflection provides a way to dynamically interact with Rust types, such as accessing fields
//! by their string name. Reflection is a core part of Bevy and enables a number of interesting
//! features (like scenes).
//!
//! Reflection is like giving your code X-ray vision - you can look inside types at runtime,
//! see their structure, and manipulate them dynamically! Just as physicists use particle
//! detectors to understand matter's structure, reflection lets us probe and modify data
//! structures while the program runs. This powers Bevy's scene system, editor tools, and
//! serialization magic.

use bevy::{
    prelude::*,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        DynamicStruct, PartialReflect,
    },
};
use serde::de::DeserializeSeed;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Bar will be automatically registered as it's a dependency of Foo
        .register_type::<Foo>()
        .add_systems(Startup, setup)
        .run();
}

/// Deriving `Reflect` implements the relevant reflection traits. In this case, it implements the
/// `Reflect` trait and the `Struct` trait `derive(Reflect)` assumes that all fields also implement
/// Reflect.
///
/// All fields in a reflected item will need to be `Reflect` as well. You can opt a field out of
/// reflection by using the `#[reflect(ignore)]` attribute.
/// If you choose to ignore a field, you need to let the automatically-derived `FromReflect` implementation
/// how to handle the field.
/// To do this, you can either define a `#[reflect(default = "...")]` attribute on the ignored field, or
/// opt-out of `FromReflect`'s auto-derive using the `#[reflect(from_reflect = false)]` attribute.
///
/// Think of #[derive(Reflect)] as installing a control panel on your struct - it creates
/// knobs and dials for every field! The "Reflect" trait is like a universal adapter that
/// lets Bevy's systems plug into your types. Every field must also be reflectable, just
/// like every component in a machine must be accessible for maintenance.
#[derive(Reflect)]
#[reflect(from_reflect = false)]  // We can't auto-generate FromReflect due to ignored field
pub struct Foo {
    a: usize,              // Simple reflected field
    nested: Bar,           // Nested structs work too!
    #[reflect(ignore)]     // This field is "invisible" to reflection
    _ignored: NonReflectedValue,  // Like a locked room in our data structure
}

/// This `Bar` type is used in the `nested` field of the `Foo` type. We must derive `Reflect` here
/// too (or ignore it)
/// 
/// Reflection is contagious - if a parent type is reflectable, all its field types must be too!
/// It's like a chain reaction: once you start reflecting, everything connected must reflect.
#[derive(Reflect)]
pub struct Bar {
    b: usize,
}

#[derive(Default)]
struct NonReflectedValue {
    _a: usize,
}

fn setup(type_registry: Res<AppTypeRegistry>) {
    let mut value = Foo {
        a: 1,
        _ignored: NonReflectedValue { _a: 10 },
        nested: Bar { b: 8 },
    };

    // FIELD ACCESS BY NAME - Like using a key instead of knowing the lock!
    // You can set field values dynamically. The type must match exactly or this will fail.
    *value.get_field_mut("a").unwrap() = 2usize;
    assert_eq!(value.a, 2);
    assert_eq!(*value.get_field::<usize>("a").unwrap(), 2);

    // DYNAMIC TYPE INTROSPECTION - Looking at data through different lenses
    // You can also get the `&dyn PartialReflect` value of a field like this
    let field = value.field("a").unwrap();

    // PartialReflect vs Reflect is like "maybe reflectable" vs "definitely reflectable"
    // But values introspected via `PartialReflect` will not return `dyn Reflect` trait objects
    // (even if the containing type does implement `Reflect`), so we need to convert them:
    let fully_reflected_field = field.try_as_reflect().unwrap();

    // DOWNCASTING - Converting from generic reflection back to concrete type
    // Like revealing what's inside a wrapped package!
    // Now, you can downcast your `Reflect` value like this:
    assert_eq!(*fully_reflected_field.downcast_ref::<usize>().unwrap(), 2);

    // For this specific case, we also support the shortcut `try_downcast_ref`:
    assert_eq!(*field.try_downcast_ref::<usize>().unwrap(), 2);

    // DYNAMIC STRUCTS - Building structures at runtime!
    // `DynamicStruct` is like a struct built from Lego blocks - assembled on the fly
    let mut patch = DynamicStruct::default();
    patch.insert("a", 4usize);

    // PATCHING - Applying changes like a software update
    // You can "apply" Reflect implementations on top of other Reflect implementations.
    // This will only set fields with the same name, and it will fail if the types don't match.
    // You can use this to "patch" your types with new values.
    value.apply(&patch);  // Only updates field "a", leaves others untouched
    assert_eq!(value.a, 4);

    // SERIALIZATION - Converting structs to text (like freeze-drying food!)
    let type_registry = type_registry.read();
    // By default, all derived `Reflect` types can be Serialized using serde. No need to derive
    // Serialize! Reflection gives you serialization for free - it's like getting a printer
    // with your scanner!
    let serializer = ReflectSerializer::new(&value, &type_registry);
    let ron_string =
        ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default()).unwrap();
    info!("{}\n", ron_string);  // RON = Rusty Object Notation, human-readable format

    // DESERIALIZATION - Reconstituting data from text (just add Bevy!)
    // Dynamic properties can be deserialized
    let reflect_deserializer = ReflectDeserializer::new(&type_registry);
    let mut deserializer = ron::de::Deserializer::from_str(&ron_string).unwrap();
    let reflect_value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    // DYNAMIC TYPES - The shapeshifters of the reflection world
    // Deserializing returns a `Box<dyn PartialReflect>` value.
    // Generally, deserializing a value will return the "dynamic" variant of a type.
    // For example, deserializing a struct will return the DynamicStruct type.
    // "Opaque types" will be deserialized as themselves.
    // Think of it like getting a generic mold that can become any shape!
    assert_eq!(
        reflect_value.reflect_type_path(),
        DynamicStruct::type_path(),
    );

    // STRUCTURAL EQUALITY - Comparing blueprints, not materials
    // Reflect has its own `partial_eq` implementation, named `reflect_partial_eq`. This behaves
    // like normal `partial_eq`, but it treats "dynamic" and "non-dynamic" types the same. The
    // `Foo` struct and deserialized `DynamicStruct` are considered equal for this reason:
    // It's like saying a house and its blueprint are "equal" if they have the same layout!
    assert!(reflect_value.reflect_partial_eq(&value).unwrap());

    // ROUND-TRIP SERIALIZATION - The complete journey
    // By "patching" `Foo` with the deserialized DynamicStruct, we can "Deserialize" Foo.
    // This means we can serialize and deserialize with a single `Reflect` derive!
    // It's like having a universal save/load system for your entire game state!
    value.apply(&*reflect_value);  // Apply the deserialized data back to original
}
