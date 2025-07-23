//! This example demonstrates the use of dynamic types in Bevy's reflection system.
//!
//! Dynamic types are like shape-shifting containers that can become any type at runtime!
//! Imagine having a universal adapter that can plug into any socket - that's what dynamic
//! types do for data. While concrete types are like specific LEGO blocks with fixed shapes,
//! dynamic types are like moldable clay that can take the form of any LEGO block when needed.
//! This enables powerful serialization, editing tools, and runtime type manipulation.

use bevy::reflect::{
    reflect_trait, serde::TypedReflectDeserializer, std_traits::ReflectDefault, DynamicArray,
    DynamicEnum, DynamicList, DynamicMap, DynamicSet, DynamicStruct, DynamicTuple,
    DynamicTupleStruct, DynamicVariant, FromReflect, PartialReflect, Reflect, ReflectFromReflect,
    Set, TypeRegistry, Typed,
};
use serde::de::DeserializeSeed;
use std::collections::{HashMap, HashSet};

fn main() {
    #[derive(Reflect, Default, PartialEq, Debug)]
    #[reflect(Identifiable, Default)]
    struct Player {
        id: u32,
    }

    #[reflect_trait]
    trait Identifiable {
        fn id(&self) -> u32;
    }

    impl Identifiable for Player {
        fn id(&self) -> u32 {
            self.id
        }
    }

    // CONCRETE TYPES - The normal, everyday way of creating data
    // Normally, when instantiating a type, you get back exactly that type.
    // This is because the type is known at compile time.
    // We call this the "concrete" or "canonical" type.
    // Like ordering a specific pizza - you know exactly what you're getting!
    let player: Player = Player { id: 123 };

    // TYPE ERASURE - Hiding the specific type behind a universal interface
    // When working with reflected types, however, we often "erase" this type information
    // using the `Reflect` trait object.
    // This trait object also gives us access to all the methods in the `PartialReflect` trait too.
    // The underlying type is still the same (in this case, `Player`),
    // but now we've hidden that information from the compiler.
    // Like putting your pizza in an opaque box - it's still pizza, but you can't see the type!
    let reflected: Box<dyn Reflect> = Box::new(player);

    // DOWNCASTING - Peeking inside the opaque box to see what type it really is
    // Because it's the same type under the hood, we can still downcast it back to the original type.
    assert!(reflected.downcast_ref::<Player>().is_some());

    // REFLECTIVE CLONING - Copying without knowing the exact type
    // We can attempt to clone our value using `PartialReflect::reflect_clone`.
    // This will recursively call `PartialReflect::reflect_clone` on all fields of the type.
    // Or, if we had registered `ReflectClone` using `#[reflect(Clone)]`, it would simply call `Clone::clone` directly.
    // Like photocopying a document without knowing what's written on it!
    let cloned: Box<dyn Reflect> = reflected.reflect_clone().unwrap();
    assert_eq!(cloned.downcast_ref::<Player>(), Some(&Player { id: 123 }));

    // DYNAMIC CONVERSION - Creating a shapeshifting proxy of our data
    // Another way we can "clone" our data is by converting it to a dynamic type.
    // Notice here we bind it as a `dyn PartialReflect` instead of `dyn Reflect`.
    // This is because it returns a dynamic type that simply represents the original type.
    // In this case, because `Player` is a struct, it will return a `DynamicStruct`.
    // Like making a clay model of your pizza - same shape and contents, different material!
    let dynamic: Box<dyn PartialReflect> = reflected.to_dynamic();
    assert!(dynamic.is_dynamic());

    // THE DYNAMIC LIMITATION - Shapeshifters can't pretend to be the original
    // And if we try to convert it back to a `dyn Reflect` trait object, we'll get `None`.
    // Dynamic types cannot be directly cast to `dyn Reflect` trait objects.
    // The clay model looks like pizza but can't be eaten like pizza!
    assert!(dynamic.try_as_reflect().is_none());

    // PROXY ACCESS - Using the dynamic type to examine structure
    // Generally dynamic types are used to represent (or "proxy") the original type,
    // so that we can continue to access its fields and overall structure.
    // Like using the clay model to study the pizza's ingredients and shape
    let dynamic_ref = dynamic.reflect_ref().as_struct().unwrap();
    let id = dynamic_ref.field("id").unwrap().try_downcast_ref::<u32>();
    assert_eq!(id, Some(&123));

    // RUNTIME TYPE CONSTRUCTION - Building types from blueprints
    // It also enables us to create a representation of a type without having compile-time
    // access to the actual type. This is how the reflection deserializers work.
    // They generally can't know how to construct a type ahead of time,
    // so they instead build and return these dynamic representations.
    // Like a factory that can build any product from a description!
    let input = "(id: 123)";
    let mut registry = TypeRegistry::default();
    registry.register::<Player>();  // Register the blueprint
    let registration = registry.get(std::any::TypeId::of::<Player>()).unwrap();
    let deserialized = TypedReflectDeserializer::new(registration, &registry)
        .deserialize(&mut ron::Deserializer::from_str(input).unwrap())
        .unwrap();

    // PROXY VALIDATION - Confirming what type this dynamic represents
    // Our deserialized output is a `DynamicStruct` that proxies/represents a `Player`.
    assert!(deserialized.represents::<Player>());

    // THE PROXY LIMITATION - Sometimes you need the real thing, not a model
    // And while this does allow us to access the fields and structure of the type,
    // there may be instances where we need the actual type.
    // For example, if we want to convert our `dyn Reflect` into a `dyn Identifiable`,
    // we can't use the `DynamicStruct` proxy.
    // Like trying to use a clay pizza to satisfy hunger - the shape is right but it's not food!
    let reflect_identifiable = registration
        .data::<ReflectIdentifiable>()
        .expect("`ReflectIdentifiable` should be registered");

    // FAILED CONVERSION - The proxy can't do everything the original can
    // Trying to access the registry with our `deserialized` will give a compile error
    // since it doesn't implement `Reflect`, only `PartialReflect`.
    // Similarly, trying to force the operation will fail.
    // This fails since the underlying type of `deserialized` is `DynamicStruct` and not `Player`.
    // The clay pizza model can't be used in recipes that need real pizza!
    assert!(deserialized
        .try_as_reflect()
        .and_then(|reflect_trait_obj| reflect_identifiable.get(reflect_trait_obj))
        .is_none());

    // THE TRANSFORMATION CHALLENGE - Converting clay back to the real thing
    // So how can we go from a dynamic type to a concrete type?
    // There are two ways:

    // METHOD 1: APPLYING DYNAMIC DATA - Copying from clay model to real object
    // 1. Using `PartialReflect::apply`.
    {
        // COMPILE-TIME KNOWN TYPES - When you know what you're making
        // If you know the type at compile time, you can construct a new value and apply the dynamic
        // value to it.
        // Like making a real pizza using the clay model as a reference!
        let mut value = Player::default();
        value.apply(deserialized.as_ref());
        assert_eq!(value.id, 123);

        // RUNTIME CONSTRUCTION - When you don't know what you're making until runtime
        // If you don't know the type at compile time, you need a dynamic way of constructing
        // an instance of the type. One such way is to use the `ReflectDefault` type data.
        // Like having a factory that can make any product from its blueprint!
        let reflect_default = registration
            .data::<ReflectDefault>()
            .expect("`ReflectDefault` should be registered");

        let mut value: Box<dyn Reflect> = reflect_default.default();  // Make blank copy
        value.apply(deserialized.as_ref());  // Copy from clay model

        let identifiable: &dyn Identifiable = reflect_identifiable.get(value.as_reflect()).unwrap();
        assert_eq!(identifiable.id(), 123);
    }

    // METHOD 2: DIRECT CONVERSION - Transforming clay into the real thing
    // 2. Using `FromReflect`
    {
        // COMPILE-TIME CONVERSION - When you know exactly what to convert to
        // If you know the type at compile time, you can use the `FromReflect` trait to convert the
        // dynamic value into the concrete type directly.
        // Like using a magic spell to turn clay pizza into real pizza!
        let value: Player = Player::from_reflect(deserialized.as_ref()).unwrap();
        assert_eq!(value.id, 123);

        // RUNTIME CONVERSION - Dynamic transformation magic
        // If you don't know the type at compile time, you can use the `ReflectFromReflect` type data
        // to perform the conversion dynamically.
        // Like having a universal transformation device that can turn any clay model into the real thing!
        let reflect_from_reflect = registration
            .data::<ReflectFromReflect>()
            .expect("`ReflectFromReflect` should be registered");

        let value: Box<dyn Reflect> = reflect_from_reflect
            .from_reflect(deserialized.as_ref())
            .unwrap();
        let identifiable: &dyn Identifiable = reflect_identifiable.get(value.as_reflect()).unwrap();
        assert_eq!(identifiable.id(), 123);
    }

    // MANUAL CONSTRUCTION - Building dynamic types by hand
    // Lastly, while dynamic types are commonly generated via reflection methods like
    // `PartialReflect::to_dynamic` or via the reflection deserializers,
    // you can also construct them manually.
    // Like hand-sculpting clay instead of using a mold!
    let mut my_dynamic_list = DynamicList::from_iter([1u32, 2u32, 3u32]);

    // PARTIAL UPDATES - Apply only the changes you want
    // This is useful when you just need to apply some subset of changes to a type.
    // Like using a stencil to paint only certain parts of a wall!
    let mut my_list: Vec<u32> = Vec::new();
    my_list.apply(&my_dynamic_list);
    assert_eq!(my_list, vec![1, 2, 3]);

    // TYPE REPRESENTATION - Teaching the dynamic type what it represents
    // And if you want it to actually proxy a type, you can configure it to do that as well:
    // Like putting a label on your clay sculpture saying "This represents a pizza"
    assert!(!my_dynamic_list
        .as_partial_reflect()
        .represents::<Vec<u32>>());
    my_dynamic_list.set_represented_type(Some(<Vec<u32>>::type_info()));  // Add the label
    assert!(my_dynamic_list
        .as_partial_reflect()
        .represents::<Vec<u32>>());

    // ============================= DYNAMIC TYPE CATALOG ============================= //
    // THE COMPLETE SHAPESHIFTER COLLECTION - Every dynamic type available!
    // For reference, here are all the available dynamic types:

    // 1. `DynamicTuple` - Ordered collections with fixed size
    {
        // Like a numbered list where position matters: (first, second, third)
        let mut dynamic_tuple = DynamicTuple::default();
        dynamic_tuple.insert(1u32);  // Position 0
        dynamic_tuple.insert(2u32);  // Position 1
        dynamic_tuple.insert(3u32);  // Position 2

        let mut my_tuple: (u32, u32, u32) = (0, 0, 0);
        my_tuple.apply(&dynamic_tuple);
        assert_eq!(my_tuple, (1, 2, 3));
    }

    // 2. `DynamicArray` - Fixed-size collections with indexed access
    {
        // Like a row of numbered parking spaces - size never changes
        let dynamic_array = DynamicArray::from_iter([1u32, 2u32, 3u32]);

        let mut my_array = [0u32; 3];
        my_array.apply(&dynamic_array);
        assert_eq!(my_array, [1, 2, 3]);
    }

    // 3. `DynamicList` - Variable-size collections that can grow and shrink
    {
        // Like a flexible shopping list - add or remove items as needed
        let dynamic_list = DynamicList::from_iter([1u32, 2u32, 3u32]);

        let mut my_list: Vec<u32> = Vec::new();
        my_list.apply(&dynamic_list);
        assert_eq!(my_list, vec![1, 2, 3]);
    }

    // 4. `DynamicSet` - Collections of unique items with no duplicates
    {
        // Like a bag of unique marbles - each marble can only appear once
        let mut dynamic_set = DynamicSet::from_iter(["x", "y", "z"]);
        assert!(dynamic_set.contains(&"x"));

        dynamic_set.remove(&"y");  // Remove the "y" marble

        let mut my_set: HashSet<&str> = HashSet::default();
        my_set.apply(&dynamic_set);
        assert_eq!(my_set, HashSet::from_iter(["x", "z"]));
    }

    // 5. `DynamicMap` - Key-value pairs like a dictionary or phone book
    {
        // Like a locker system - each key opens one specific locker with a value inside
        let dynamic_map = DynamicMap::from_iter([("x", 1u32), ("y", 2u32), ("z", 3u32)]);

        let mut my_map: HashMap<&str, u32> = HashMap::default();
        my_map.apply(&dynamic_map);
        assert_eq!(my_map.get("x"), Some(&1));  // Key "x" opens locker with value 1
        assert_eq!(my_map.get("y"), Some(&2));
        assert_eq!(my_map.get("z"), Some(&3));
    }

    // 6. `DynamicStruct` - Named fields like a form with labeled boxes
    {
        #[derive(Reflect, Default, Debug, PartialEq)]
        struct MyStruct {
            x: u32,
            y: u32,
            z: u32,
        }

        // Like filling out a form - each field has a name and a value
        let mut dynamic_struct = DynamicStruct::default();
        dynamic_struct.insert("x", 1u32);  // Fill the "x" field
        dynamic_struct.insert("y", 2u32);  // Fill the "y" field
        dynamic_struct.insert("z", 3u32);  // Fill the "z" field

        let mut my_struct = MyStruct::default();
        my_struct.apply(&dynamic_struct);
        assert_eq!(my_struct, MyStruct { x: 1, y: 2, z: 3 });
    }

    // 7. `DynamicTupleStruct` - Numbered fields like a struct with positions instead of names
    {
        #[derive(Reflect, Default, Debug, PartialEq)]
        struct MyTupleStruct(u32, u32, u32);

        // Like a form with numbered boxes instead of named fields: Box 0, Box 1, Box 2
        let mut dynamic_tuple_struct = DynamicTupleStruct::default();
        dynamic_tuple_struct.insert(1u32);  // Box 0
        dynamic_tuple_struct.insert(2u32);  // Box 1
        dynamic_tuple_struct.insert(3u32);  // Box 2

        let mut my_tuple_struct = MyTupleStruct::default();
        my_tuple_struct.apply(&dynamic_tuple_struct);
        assert_eq!(my_tuple_struct, MyTupleStruct(1, 2, 3));
    }

    // 8. `DynamicEnum` - Choice between different variants, like a multiple choice question
    {
        #[derive(Reflect, Default, Debug, PartialEq)]
        enum MyEnum {
            #[default]
            Empty,           // Choice A: Nothing
            Xyz(u32, u32, u32),  // Choice B: Three numbers
        }

        // Build the data for Choice B
        let mut values = DynamicTuple::default();
        values.insert(1u32);
        values.insert(2u32);
        values.insert(3u32);

        // Create the enum variant and select "Xyz" option
        let dynamic_variant = DynamicVariant::Tuple(values);
        let dynamic_enum = DynamicEnum::new("Xyz", dynamic_variant);

        let mut my_enum = MyEnum::default();
        my_enum.apply(&dynamic_enum);
        assert_eq!(my_enum, MyEnum::Xyz(1, 2, 3));
    }
}
