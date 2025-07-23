//! # Custom Query Parameters in Bevy ECS
//!
//! This example demonstrates how to create custom query types using the [`QueryData`] derive macro.
//! Instead of using tuples like `Query<(&ComponentA, &ComponentB, Entity)>`, you can define
//! structured query types that are more maintainable and self-documenting.
//!
//! ## Why Use Custom Query Types?
//!
//! While regular tuple queries work great in most simple scenarios, custom queries provide:
//!
//! 1. **Named Fields**: Access components with `query_item.position` instead of `query_item.2`
//! 2. **Easy Refactoring**: Add/remove components without updating destructuring patterns everywhere
//! 3. **Composition**: Build complex queries from smaller, reusable query types  
//! 4. **No Tuple Limits**: Bypass the 15-component limit that exists for tuple queries
//! 5. **Better Documentation**: Query structs can have doc comments on individual fields
//!
//! ## Key Concepts Demonstrated
//!
//! - Creating read-only and mutable custom query types
//! - Using generics in custom queries for flexibility
//! - Composing queries by nesting other query types
//! - Creating custom query filters
//! - Handling optional components in custom queries
//!
//! For more details on the [`QueryData`] derive macro, see the trait documentation.

use bevy::{
    // The QueryData derive macro generates the boilerplate for custom query types
    // QueryFilter is used to create custom filters (like With, Without, etc.)
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};
// Debug trait is needed to print our components in the example
use std::fmt::Debug;

fn main() {
    App::new()
        // Spawn entities with our components during startup
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            // Chain these systems to run sequentially, demonstrating different query styles
            (
                print_components_read_only,  // Uses a read-only custom query
                print_components_iter_mut,   // Uses a mutable custom query  
                print_components_iter,       // Shows read-only access to mutable query
                print_components_tuple,      // Traditional tuple query for comparison
            )
                // The .chain() method ensures these systems run in order, one after another
                // This is useful when you want to see output in a specific sequence
                .chain(),
        )
        .run();
}

// Simple marker components for demonstration
// In a real game, these might be Position, Velocity, Health, etc.
// The Debug derive allows us to print these components
#[derive(Component, Debug)]
struct ComponentA;

#[derive(Component, Debug)]
struct ComponentB;

#[derive(Component, Debug)]
struct ComponentC;

#[derive(Component, Debug)]
struct ComponentD;

// ComponentZ is intentionally NOT added to any entities
// This demonstrates how filters work with absent components
#[derive(Component, Debug)]
struct ComponentZ;

/// A read-only custom query that demonstrates various query features.
/// The QueryData derive macro generates all the necessary trait implementations.
#[derive(QueryData)]
// This attribute adds Debug to the generated query item types
#[query_data(derive(Debug))]
struct ReadOnlyCustomQuery<T: Component + Debug, P: Component + Debug> {
    // Access the entity ID - useful for commands or identification
    entity: Entity,
    
    // Required component - query will only match entities with ComponentA
    // The 'static lifetime is automatically inferred by Bevy and ensures safety
    a: &'static ComponentA,
    
    // Optional component - wrapped in Option<T>
    // Query matches entities with or without ComponentB
    b: Option<&'static ComponentB>,
    
    // Nested query - demonstrates composition pattern
    // NestedQuery is another QueryData type defined below
    nested: NestedQuery,
    
    // Optional nested query - only matches if ALL components in NestedQuery exist
    optional_nested: Option<NestedQuery>,
    
    // Optional tuple of components - both must exist for Some, otherwise None
    optional_tuple: Option<(&'static ComponentB, &'static ComponentZ)>,
    
    // Generic query components - allows this query to be reused with different types
    // T and P are constrained to be Components with Debug for printing
    generic: GenericQuery<T, P>,
    
    // Empty query - always matches, useful for certain patterns
    empty: EmptyQuery,
}

/// System that demonstrates using a read-only custom query.
/// Note how we access fields by name rather than tuple indices.
fn print_components_read_only(
    // Query takes two type parameters:
    // 1. The QueryData type (what components to fetch)
    // 2. The QueryFilter type (what entities to include/exclude)
    query: Query<
        // Concrete type parameters for our generic query
        ReadOnlyCustomQuery<ComponentC, ComponentD>,
        // Custom filter that requires certain components
        CustomQueryFilter<ComponentC, ComponentD>,
    >,
) {
    println!("Print components (read_only):");
    
    // Iterate over all entities matching our query and filter
    // The & means we're borrowing immutably (read-only access)
    for e in &query {
        // Access components using named fields - much clearer than e.0, e.1, etc.
        println!("Entity: {}", e.entity);
        println!("A: {:?}", e.a);
        println!("B: {:?}", e.b);
        println!("Nested: {:?}", e.nested);
        println!("Optional nested: {:?}", e.optional_nested);
        println!("Optional tuple: {:?}", e.optional_tuple);
        println!("Generic: {:?}", e.generic);
    }
    println!();
}

/// A mutable custom query that allows modifying components.
/// 
/// ## Key Points:
/// - The `mutable` attribute tells QueryData to generate mutable access code
/// - This actually generates TWO types:
///   1. `CustomQuery` - The mutable version (with &mut references)
///   2. `CustomQueryReadOnly` - An immutable version (with & references)
/// - You can use either version depending on your needs
/// 
/// Note: if you want to use derive macros with read-only query variants, 
/// you need to pass them using the `derive` attribute.
#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
struct CustomQuery<T: Component + Debug, P: Component + Debug> {
    // Same as read-only version
    entity: Entity,
    
    // Mutable reference - allows modifying ComponentA
    a: &'static mut ComponentA,
    
    // Optional mutable reference
    b: Option<&'static mut ComponentB>,
    
    // Nested queries can be read-only even in mutable queries
    nested: NestedQuery,
    
    // Optional nested query
    optional_nested: Option<NestedQuery>,
    
    // Mix of read-only (NestedQuery) and mutable (&mut ComponentZ) access
    optional_tuple: Option<(NestedQuery, &'static mut ComponentZ)>,
    
    // Generic components
    generic: GenericQuery<T, P>,
    
    // Empty query
    empty: EmptyQuery,
}

/// An empty query that matches all entities.
/// 
/// This pattern is useful when:
/// - You need to count all entities
/// - You're using filters to restrict matches
/// - You're composing this into a larger query
#[derive(QueryData)]
#[query_data(derive(Debug))]
struct EmptyQuery {
    // The unit type () means "no data"
    // This query matches every entity but doesn't fetch any components
    empty: (),
}

/// A simple query struct that can be nested inside other queries.
/// This demonstrates the composition pattern - building complex queries from simple ones.
#[derive(QueryData)]
#[query_data(derive(Debug))]
struct NestedQuery {
    // Required component
    c: &'static ComponentC,
    // Optional component
    d: Option<&'static ComponentD>,
}

/// A generic query that works with any component types.
/// 
/// Generics in queries allow:
/// - Code reuse across different component types
/// - Type safety at compile time
/// - Flexibility in system design
#[derive(QueryData)]
#[query_data(derive(Debug))]
struct GenericQuery<T: Component, P: Component> {
    // Tuple of generic components
    // Both T and P must be present for this to match
    generic: (&'static T, &'static P),
}

/// A custom filter that combines multiple filtering conditions.
/// 
/// Filters determine which entities a query will match, beyond just
/// having the required components. Common filters include:
/// - `With<T>`: Entity must have component T
/// - `Without<T>`: Entity must NOT have component T  
/// - `Added<T>`: Component T was added this frame
/// - `Changed<T>`: Component T was modified this frame
/// - `Or<(...)>`: At least one condition in the tuple must be true
#[derive(QueryFilter)]
struct CustomQueryFilter<T: Component, P: Component> {
    // Underscore prefix is a Rust convention for "unused" fields
    // These fields exist only for their filtering behavior
    
    // Entity must have ComponentC
    _c: With<ComponentC>,
    
    // Entity must have ComponentD
    _d: With<ComponentD>,
    
    // At least ONE of these must be true:
    // - ComponentC was just added
    // - ComponentD was changed
    // - Entity does NOT have ComponentZ
    _or: Or<(Added<ComponentC>, Changed<ComponentD>, Without<ComponentZ>)>,
    
    // Entity must have both generic components
    _generic_tuple: (With<T>, With<P>),
}

/// Startup system that spawns an entity with multiple components.
/// Note that we're NOT adding ComponentZ - this helps demonstrate
/// how the Without<ComponentZ> filter works.
fn spawn(mut commands: Commands) {
    // Spawn a single entity with a bundle of components
    // The tuple syntax is a shorthand for adding multiple components at once
    commands.spawn((ComponentA, ComponentB, ComponentC, ComponentD));
}

/// System demonstrating mutable iteration over a custom query.
/// The `mut` keyword appears in three places:
/// 1. `mut query` - The Query itself needs to be mutable
/// 2. `&mut query` - We iterate mutably
/// 3. The components we get back are mutable references
fn print_components_iter_mut(
    mut query: Query<
        CustomQuery<ComponentC, ComponentD>,
        CustomQueryFilter<ComponentC, ComponentD>,
    >,
) {
    println!("Print components (iter_mut):");
    
    // &mut iteration gives us mutable access to components
    for e in &mut query {
        // This line shows the actual type we get from iteration
        // CustomQueryItem is the generated type for mutable access
        // The underscores let Rust infer the generic parameters
        let e: CustomQueryItem<'_, _, _> = e;
        
        // Even though we have mutable access, we're just reading here
        // In a real system, you might do: *e.a = ComponentA::new()
        println!("Entity: {}", e.entity);
        println!("A: {:?}", e.a);
        println!("B: {:?}", e.b);
        println!("Optional nested: {:?}", e.optional_nested);
        println!("Optional tuple: {:?}", e.optional_tuple);
        println!("Nested: {:?}", e.nested);
        println!("Generic: {:?}", e.generic);
    }
    println!();
}

/// System showing that you can use a mutable query type for read-only access.
/// When you iterate with & instead of &mut, you get the read-only version.
fn print_components_iter(
    // Note: query is NOT mut here, even though CustomQuery is a mutable query type
    query: Query<CustomQuery<ComponentC, ComponentD>, CustomQueryFilter<ComponentC, ComponentD>>,
) {
    println!("Print components (iter):");
    
    // & iteration on a mutable query type gives read-only access
    for e in &query {
        // CustomQueryReadOnlyItem is the automatically generated read-only version
        // This is created because we used #[query_data(mutable)] on CustomQuery
        let e: CustomQueryReadOnlyItem<'_, _, _> = e;
        
        // All component access is immutable here
        println!("Entity: {}", e.entity);
        println!("A: {:?}", e.a);
        println!("B: {:?}", e.b);
        println!("Nested: {:?}", e.nested);
        println!("Generic: {:?}", e.generic);
    }
    println!();
}

// Type aliases make complex tuple queries more readable
// The 'w lifetime represents the "world" borrow lifetime
// This is how long the references to components are valid
type NestedTupleQuery<'w> = (&'w ComponentC, &'w ComponentD);
type GenericTupleQuery<'w, T, P> = (&'w T, &'w P);

/// Traditional tuple-based query for comparison with custom queries.
/// This shows what we'd write without the QueryData derive macro.
fn print_components_tuple(
    query: Query<
        // Query data as a tuple - gets unwieldy with many components
        (
            Entity,
            &ComponentA,
            &ComponentB,
            NestedTupleQuery,
            GenericTupleQuery<ComponentC, ComponentD>,
        ),
        // Filter as a tuple - same filters as CustomQueryFilter
        (
            With<ComponentC>,
            With<ComponentD>,
            Or<(Added<ComponentC>, Changed<ComponentD>, Without<ComponentZ>)>,
        ),
    >,
) {
    println!("Print components (tuple):");
    
    // Destructuring syntax extracts all components from the tuple
    // With many components, this line gets very long and error-prone
    for (entity, a, b, nested, (generic_c, generic_d)) in &query {
        println!("Entity: {entity}");
        println!("A: {a:?}");
        println!("B: {b:?}");
        
        // Accessing tuple elements by index (.0, .1) is less readable
        // than named fields (nested.c, nested.d)
        println!("Nested: {:?} {:?}", nested.0, nested.1);
        println!("Generic: {generic_c:?} {generic_d:?}");
    }
}
