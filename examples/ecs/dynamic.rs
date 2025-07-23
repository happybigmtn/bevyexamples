//! # Dynamic Components in Bevy ECS
//!
//! This advanced example demonstrates how to create and use components dynamically at runtime,
//! bypassing Rust's static type system. This is useful for:
//! - Scripting systems where component types aren't known at compile time
//! - Modding support where users can define new component types
//! - Data-driven architectures where components are defined in configuration files
//!
//! ## Key Concepts Covered
//!
//! - Creating components without defining Rust types
//! - Using `ComponentId` instead of concrete types
//! - Building queries dynamically with `QueryBuilder`
//! - Working with raw memory layouts and pointers
//! - Interactive command-line interface for ECS operations
//!
//! ## Safety Considerations
//!
//! This example uses unsafe code because:
//! - We're working with raw memory layouts
//! - We bypass Rust's type system for dynamic behavior
//! - We manually manage component data as byte arrays

// Allow unsafe code with a clear reason
#![expect(
    unsafe_code,
    reason = "Unsafe code is needed to work with dynamic components"
)]

use std::{
    // Layout describes memory layout (size and alignment) for types
    alloc::Layout,
    // HashMap to store component name -> ID mappings
    collections::HashMap,
    // For flushing stdout to ensure prompts appear
    io::Write,
    // NonNull is a non-null raw pointer with covariance
    ptr::NonNull,
};

use bevy::{
    ecs::{
        component::{
            // Controls how components behave when entities are cloned
            ComponentCloneBehavior,
            // Describes a component's properties without a concrete type
            ComponentDescriptor,
            // Unique identifier for a component type
            ComponentId,
            // Runtime information about a component
            ComponentInfo,
            // Where component data is stored (Table or SparseSet)
            StorageType,
        },
        query::{
            // Describes how a component is accessed (shared or exclusive)
            ComponentAccessKind,
            // Trait for types that can be queried
            QueryData,
        },
        // An entity reference with filtered access to components
        world::FilteredEntityMut,
    },
    prelude::*,
    ptr::{
        // Marker type indicating aligned memory
        Aligned,
        // A pointer that owns its data (for moving data into ECS)
        OwningPtr,
    },
};

// User interface prompts explaining the interactive commands

const PROMPT: &str = "
Commands:
    comp, c   Create new components
    spawn, s  Spawn entities
    query, q  Query for entities
Enter a command with no parameters for usage.";

// Component creation: defines new component types with a name and size
// Components are stored as arrays of u64 values
const COMPONENT_PROMPT: &str = "
comp, c   Create new components
    Enter a comma separated list of type names optionally followed by a size in u64s.
    e.g. CompA 3, CompB, CompC 2";

// Entity spawning: creates entities with specified components and initial values
const ENTITY_PROMPT: &str = "
spawn, s  Spawn entities
    Enter a comma separated list of components optionally followed by values.
    e.g. CompA 0 1 0, CompB, CompC 1";

// Query syntax mimics Bevy's query syntax but works with dynamic components
// - 'A' means the entity must have component A (With<A>)
// - '&A' means read-only access to component A
// - '&mut A' means mutable access to component A
// - '||' is logical OR between conditions
// - ',' is logical AND between conditions
// - '?' makes a component optional
const QUERY_PROMPT: &str = "
query, q  Query for entities
    Enter a query to fetch and update entities
    Components with read or write access will be displayed with their values
    Components with write access will have their fields incremented by one

    Accesses: 'A' with, '&A' read, '&mut A' write
    Operators: '||' or, ',' and, '?' optional

    e.g. &A || &B, &mut C, D, ?E";

fn main() {
    // Create a new ECS world to store entities and components
    let mut world = World::new();
    
    // Iterator over stdin lines for interactive input
    let mut lines = std::io::stdin().lines();
    
    // Maps component names (strings) to their runtime IDs
    // This allows us to reference components by name in commands
    let mut component_names = HashMap::<String, ComponentId>::new();
    
    // Caches component metadata for quick access during queries
    let mut component_info = HashMap::<ComponentId, ComponentInfo>::new();

    println!("{PROMPT}");
    
    // Main interactive loop
    loop {
        // Print prompt and flush to ensure it appears before input
        print!("\n> ");
        let _ = std::io::stdout().flush();
        
        // Read next line from stdin, exit on EOF or error
        let Some(Ok(line)) = lines.next() else {
            return;
        };

        // Empty line exits the program
        if line.is_empty() {
            return;
        };

        // Try to split command and arguments
        // If no arguments provided, show help for that command
        let Some((first, rest)) = line.trim().split_once(|c: char| c.is_whitespace()) else {
            // Show command-specific help based on first character
            match &line.chars().next() {
                Some('c') => println!("{COMPONENT_PROMPT}"),
                Some('s') => println!("{ENTITY_PROMPT}"),
                Some('q') => println!("{QUERY_PROMPT}"),
                _ => println!("{PROMPT}"),
            }
            continue;
        };

        // Process commands based on first character
        match &first[0..1] {
            // === Component Creation Command ===
            "c" => {
                // Parse comma-separated component definitions
                rest.split(',').for_each(|component| {
                    let mut component = component.split_whitespace();
                    
                    // Extract component name
                    let Some(name) = component.next() else {
                        return;
                    };
                    
                    // Extract optional size (defaults to 0)
                    // Size determines how many u64 values the component stores
                    let size = match component.next().map(str::parse) {
                        Some(Ok(size)) => size,
                        _ => 0,
                    };
                    
                    // Register a new dynamic component type
                    // SAFETY: We ensure [u64] is Send + Sync, which is required for components
                    let id = world.register_component_with_descriptor(unsafe {
                        ComponentDescriptor::new_with_layout(
                            name.to_string(),          // Component name for debugging
                            StorageType::Table,        // Store in tables (dense storage)
                            Layout::array::<u64>(size).unwrap(), // Memory layout as array of u64s
                            None,                      // No custom drop function
                            true,                      // Component is Send + Sync
                            ComponentCloneBehavior::Default, // Standard clone behavior
                        )
                    });
                    
                    // Get component metadata from the world
                    let Some(info) = world.components().get_info(id) else {
                        return;
                    };
                    
                    // Store mappings for later use
                    component_names.insert(name.to_string(), id);
                    component_info.insert(id, info.clone());
                    
                    println!("Component {} created with id: {}", name, id.index());
                });
            }
            // === Entity Spawning Command ===
            "s" => {
                // Vectors to collect component IDs and their data
                let mut to_insert_ids = Vec::new();
                let mut to_insert_data = Vec::new();
                
                // Parse each component specification
                rest.split(',').for_each(|component| {
                    let mut component = component.split_whitespace();
                    
                    // Get component name
                    let Some(name) = component.next() else {
                        return;
                    };

                    // Look up the component ID from our name registry
                    let Some(&id) = component_names.get(name) else {
                        println!("Component {name} does not exist");
                        return;
                    };

                    // Calculate how many u64 values this component stores
                    // based on the layout we specified when creating it
                    let info = world.components().get_info(id).unwrap();
                    let len = info.layout().size() / size_of::<u64>();
                    
                    // Parse values from the command, defaulting to 0 for missing values
                    let mut values: Vec<u64> = component
                        .take(len)  // Only take as many values as the component size
                        .filter_map(|value| value.parse::<u64>().ok())
                        .collect();
                    values.resize(len, 0);  // Pad with zeros if not enough values provided

                    // Store the component ID and its data
                    to_insert_ids.push(id);
                    to_insert_data.push(values);
                });

                // Create an empty entity
                let mut entity = world.spawn_empty();

                // Convert our Vec<u64> data into OwningPtr that ECS can use
                // This transfers ownership of the data to the ECS
                let to_insert_ptr = to_owning_ptrs(&mut to_insert_data);

                // Insert components by ID rather than by type
                // SAFETY:
                // - Component IDs are valid (came from this world)
                // - Data layout matches what we registered for each component
                // - OwningPtr transfers ownership correctly
                unsafe {
                    entity.insert_by_ids(&to_insert_ids, to_insert_ptr.into_iter());
                }

                println!("Entity spawned with id: {}", entity.id());
            }
            // === Query Command ===
            "q" => {
                // QueryBuilder allows constructing queries dynamically
                // FilteredEntityMut gives us an entity with filtered component access
                let mut builder = QueryBuilder::<FilteredEntityMut>::new(&mut world);
                
                // Parse the query string and build the query
                parse_query(rest, &mut builder, &component_names);
                
                // Build and execute the query
                let mut query = builder.build();
                
                // Iterate over all entities matching the query
                query.iter_mut(&mut world).for_each(|filtered_entity| {
                    // Build a string showing all accessed components and their values
                    let terms = filtered_entity
                        .access()
                        .try_iter_component_access()
                        .unwrap()
                        .map(|component_access| {
                            // Get component ID and metadata
                            let id = *component_access.index();
                            let ptr = filtered_entity.get_by_id(id).unwrap();
                            let info = component_info.get(&id).unwrap();
                            
                            // Calculate array length from component layout
                            let len = info.layout().size() / size_of::<u64>();

                            // Convert raw pointer to slice for access
                            // SAFETY:
                            // - All components store data as [u64] arrays
                            // - Length is calculated from registered layout
                            // - Pointer is valid for the component's lifetime
                            let data = unsafe {
                                std::slice::from_raw_parts_mut(
                                    ptr.assert_unique().as_ptr().cast::<u64>(),
                                    len,
                                )
                            };

                            // Demo behavior: increment values if we have write access
                            // ComponentAccessKind::Exclusive means &mut access
                            if matches!(component_access, ComponentAccessKind::Exclusive(_)) {
                                data.iter_mut().for_each(|data| {
                                    *data += 1;
                                });
                            }

                            // Format component name and values for display
                            format!("{}: {:?}", info.name(), data[0..len].to_vec())
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!("{}: {}", filtered_entity.id(), terms);
                });
            }
            _ => continue,
        }
    }
}

/// Converts component data (Vec<u64>) into OwningPtr for insertion into ECS.
/// 
/// OwningPtr is Bevy's way of taking ownership of dynamically-typed data.
/// The lifetime connection ensures data isn't dropped while ECS uses it.
/// 
/// # Safety
/// The returned pointers are valid only as long as `components` isn't dropped.
/// The ECS will take ownership of the data through these pointers.
fn to_owning_ptrs(components: &mut [Vec<u64>]) -> Vec<OwningPtr<Aligned>> {
    components
        .iter_mut()
        .map(|data| {
            // Get raw pointer to the Vec's data
            let ptr = data.as_mut_ptr();
            
            // SAFETY:
            // - Vec::as_mut_ptr() returns a valid, non-null pointer
            // - The data remains valid because we hold a mutable reference
            // - ECS will take ownership before `components` is dropped
            unsafe {
                // NonNull is a non-null pointer wrapper with variance
                let non_null = NonNull::new_unchecked(ptr.cast());
                // OwningPtr indicates this pointer owns its data
                OwningPtr::new(non_null)
            }
        })
        .collect()
}

/// Parses a single query term and adds it to the QueryBuilder.
/// 
/// Query terms can be:
/// - `A` - Entity must have component A (With filter)
/// - `&A` - Read access to component A
/// - `&mut A` - Write access to component A
/// - `?A` - Optional component A (doesn't filter, but accesses if present)
/// 
/// This mimics Bevy's static query syntax but works with dynamic component IDs.
fn parse_term<Q: QueryData>(
    str: &str,
    builder: &mut QueryBuilder<Q>,
    components: &HashMap<String, ComponentId>,
) {
    let mut matched = false;
    let str = str.trim();
    
    // Check first character to determine term type
    match str.chars().next() {
        // Optional term: ?ComponentName
        Some('?') => {
            // Recursively parse the term without the '?'
            // optional() makes the component not required for matching
            builder.optional(|b| parse_term(&str[1..], b, components));
            matched = true;
        }
        // Reference term: &ComponentName or &mut ComponentName
        Some('&') => {
            let mut parts = str.split_whitespace();
            let first = parts.next().unwrap();
            
            // Check if it's &mut (mutable reference)
            if first == "&mut" {
                if let Some(str) = parts.next() {
                    if let Some(&id) = components.get(str) {
                        // Add mutable access to this component
                        builder.mut_id(id);
                        matched = true;
                    }
                };
            } else if let Some(&id) = components.get(&first[1..]) {
                // It's just & (immutable reference)
                // Skip the & character and look up the component
                builder.ref_id(id);
                matched = true;
            }
        }
        // With term: ComponentName (no prefix)
        Some(_) => {
            if let Some(&id) = components.get(str) {
                // Entity must have this component, but we don't access it
                builder.with_id(id);
                matched = true;
            }
        }
        None => {}
    };

    if !matched {
        println!("Unable to find component: {str}");
    }
}

/// Parses a full query string and builds a dynamic query.
/// 
/// Query syntax:
/// - `,` separates AND conditions (all must match)
/// - `||` separates OR conditions (at least one must match)
/// 
/// Examples:
/// - `&A, &B` - Entities with both A and B components
/// - `&A || &B` - Entities with A or B (or both)
/// - `&A, &B || &C` - Entities with A and (B or C)
fn parse_query<Q: QueryData>(
    str: &str,
    builder: &mut QueryBuilder<Q>,
    components: &HashMap<String, ComponentId>,
) {
    // Split by commas for AND conditions
    let str = str.split(',');
    
    str.for_each(|term| {
        // Check if this term contains OR conditions
        let sub_terms: Vec<_> = term.split("||").collect();
        
        if sub_terms.len() == 1 {
            // Simple term, no OR conditions
            parse_term(sub_terms[0], builder, components);
        } else {
            // Multiple terms with OR - at least one must match
            builder.or(|b| {
                sub_terms
                    .iter()
                    .for_each(|term| parse_term(term, b, components));
            });
        }
    });
}
