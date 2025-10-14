
# Generic Cache System (Rust)

this is a clear version of project after a lot of trial and error methods. yes, took help of AI for debugging and making it clear. also, if you're searching for real messy versions of the same project, you can find them in my rust repo (which is private as of now but soon to be enclosed).

This project is a simple but complete generic in-memory cache system written in Rust.  
It was built step by step after many attempts and debugging sessions, mainly to understand how traits, generics, and data handling work in real-world code.  
The entire logic is inside a single `main.rs` file to keep it easy to follow and execute directly.

The cache system demonstrates how to use generics, traits, lifetimes, and iterators together in one project.  
It also shows how to manage data using HashMaps, track statistics, and apply time-based expiration (TTL) to cache entries.

---

## What the Project Does

The program stores data temporarily in memory and retrieves it faster the next time the same key is requested.  
It simulates how real caching systems work — remembering results to avoid recomputing or re-fetching.

It includes:
- Basic in-memory cache with key-value pairs
- Optional TTL (time-to-live) for automatic expiration
- Hit and miss tracking for statistics
- An iterator to loop through all valid cache entries
- A computed cache feature that stores results of expensive operations
- A small Least Recently Used (LRU) cache example

---

## Concepts Covered

This project mainly focuses on:
- **Traits**: defining generic behavior that can be reused for different data types  
- **Generics**: making the cache system flexible for any key-value pair type  
- **Lifetimes**: managing data references safely  
- **Iterators**: implementing custom iteration logic  
- **HashMap** usage: efficient key-value storage  
- **Structs and Methods**: organizing related data and functionality  
- **Option and Result types**: handling missing or expired data safely

---

## Project Setup

### Step 1: Create the Project
```bash
cargo new generic_cache_system
```

### Step 2 Project structure

```
generic_cache_system/
├─ Cargo.toml
└─ src/
   └─ main.rs
```

### Step 3 Dependencies

No external crates are required for the main logic.
However, you can later add optional crates like chrono for better time handling or lru for an advanced LRU cache.

```
[dependencies]
# optional
# chrono = "0.4"
# lru = "0.12"
```

### Step 4 Run project
``` 
cargo run 
```
Thankyou.... ChandanaMandarapu 