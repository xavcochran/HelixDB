# HelixDB

HelixDB is a lightweight graph database with lightning fast startup times, and millisecond query latency.

## Intial Features

- **Light Weight**: Helix will be designed to be as lightweight as possible. This will allow for reduced costs due to the lower hardware requirements, and faster startup times.
- **Fast Startup**: Helix will be designed to have lightning fast startup times through its optimised data storage format, and lazy loading.
- **Millisecond Query Latency**: Helix will have millisecond query latency through its optimised query engine and lightweight design. 
- **Highly Available**: Helix will be designed to be highly available. This will allow for reduced downtime, and increased reliability.
- **Scalable**: Helix will be highly scalable allowing for increased data storage, and increased query throughput.
- **Distributed**: Helix will be designed to be distributed meaning higher resiliancy, improved uptime and increased query throughput.
- **ACID Compliant**: Helix will be ACID compliant for data integrity and consistency.
- **Intelligent Data Sharding**: Helix will shard data based on importance. This will allow for for lazy loading on startup leading to lightning fast startup times.

## Future Features
- **Machine Learning**: Helix ML will use the Deep Graph Library to offer machine learning capabilities directly on top of the graph, improving the accuracy of your predicitions and reducing the development time to implement machine learning on your graph data.
- **Managed Service**: Helix will also offer a managed service so you can host Helix in the cloud without having to worry about the infrastructure.
- **On Premise**: If you need to run Helix on your own infrastructure, you will also have the option to run Helix on premise.
- **Multi Memory Options**: Helix will allow for multiple memory configuations, each with different trade-offs. For example, in-memory, disk, and hybrid.
- **New Query Language**: Helix will have a new query language that is optimised for graph queries. This will allow for faster query times, faster development times, and compile time query checking.
- **Compiled Queries**: Helix will provide open source libraries which will encode queries into a specific binary format meaning long complex string queries being send over the network will be a thing of the past. 

# Roadmap


---
## Phase 1

> #### Time Frame: 3 Months

### Goals
- [ ] Implement RocksDB as the storage engine
- [ ] Implement a basic graph engine that can store and retrieve nodes and edges
- [ ] Implement a basic code based API for testing in the language of choice




---
## Phase 2

> #### Time Frame: 6 Months

### Goals
- [ ] Implement data sharding based on node importance
- [ ] Test startup times based on this sharding to show results


---
## Phase 3

> #### Time Frame: TBC

### Goals
- [ ] Implement HTTP connections to the database.
- [ ] Implement a basic query language for the database


---
## Phase 4

> #### Time Frame: TBC

### Goals
- [ ] Expand the query language to include more complex queries
- [ ] Implement an optimising for the query engine


---
## Phase 5 and beyond

> #### Time Frame: TBC

### Goals
- [ ] Fully featured query language and query engine
- [ ] Caching layer for the database
- [ ] Multiple memory configurations
- [ ] Machine Learning capabilities
- [ ] Managed Service
- [ ] On Premise
- [ ] ACID Compliance



# Tech Stack Possibilities

- **Language**: Rust, Go, C++, Zig
- **Storage Engine**: RocksDB

#### Rust
| Pros | Cons |
|------|------|
| Memory safety without runtime cost | Can have larger binaries |
| Excellent concurrency model | Steeper learning curve |
| Strong type system (Algebraic Data Types) | Longer compile times |
| Great async support | Younger ecosystem than C++ |
| Growing ecosystem | Some features still maturing |
| Good C/C++ interop | |

#### Go
| Pros | Cons |
|------|------|
| Easy to learn | Garbage Collection (Pauses) |
| Fast compile times | Less control over memory |
| Garbage Collection | Limited Generics |
| Great concurrency model | Not ideal for systems programming |
| Good standard library | |
| Good C interop | |
| Growing ecosystem | |

#### C++
| Pros | Cons |
|------|------|
| Maximum performance control | Manual memory management |
| Mature ecosystem | More prone to bugs |
| Extensive libraries | Complex build systems |
| Proven in databases (Neo4j, Nebula) | Steeper learning curve |
| Direct hardware control | |

#### Zig
| Pros | Cons |
|------|------|
| Manual memory management | Younger ecosystem |
| Simple C integration | Smaller community |
| No hidden allocations | Less libraries |
| Small runtime | Steeper learning curve |
| Great error handling | Less help available |
| Easy to learn | |

#### RocksDB
| Pros | Cons |
|------|------|
| Industry proven | Can Have Complex Tuning |
| High performance | C++ Integration needed |
| Used in many databases | General Purpose |
| Good Documentation | Less control over storage format |



