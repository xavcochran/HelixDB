# Helix Grammar

### Needs

The Helix grammar needs to be easy to read, learn and use, while also being powerful enough to express complex queries. It should be able to be used like a standard programming language, allowing the use of complex, rich data types, and complex control flow.

Below are some examples of current query languages for the following functionality:

1. Get all the followers of a user
2. Filter out followers who are not active
3. Limit the number of followers to 50
4. Return the username, follower count, and following count of the followers

```javascript
// Example Query using Gremlin
userID = "abcd1234

g.V()
  .hasLabel('user')
  .hasId(userID)
  .in('follows')
  .where(__.has('status', 'active'))
  .range(0, 50)
  .dedup()
  .project('username', 'followerCount', 'followingCount')
  .by('username')
  .by('followerCount')
  .by('followingCount')
  .toList()
```

```cypher
// Equivalent Cypher Query
MATCH (follower:user)-[:follows]->(user:user {id: $userID})
WHERE follower.status = 'active'
WITH DISTINCT follower
LIMIT 50
RETURN follower.username as username,
       follower.followerCount as followerCount,
       follower.followingCount as followingCount;
```

```sql
// Equivalent SPARQL Query
SELECT DISTINCT ?username ?followerCount ?followingCount
WHERE {
    ?follower a :user ;
             :status "active" ;
             :username ?username ;
             :followerCount ?followerCount ;
             :followingCount ?followingCount ;
             :follows ?user .
    ?user a :user ;
          :id ?userID .
    FILTER (?userID = "abcd1234")
}
LIMIT 50
```

#### HelixQL Example

```rust
use Schema::{User}                                  // import the schema

QUERY GetFollowers(userID: String) =>               // declare new query
    GET User(userID)::In::Follows DISTINCT          // get all the followers of a user returning the user
        WHERE Status::Active                        // filter out followers who are not active
    LIMIT 50                                        // limit the number of followers to 50
    RETURN Username, FollowerCount, FollowingCount  // return the username, follower count, and following count of the followers
```

**Output:** The output of the query will automatically be transformed into the native language type unless specified otherwise.

# HelixQL

#### Schema definitions

A Helix schema enriches queries with types. 

#### `QUERY` Statement

The `QUERY` statement is used to declare a new query in HelixQL. The query name is declared after the `QUERY` keyword. The query name must be unique and and must be followed by the `=>` operator.

```rust
QUERY FunctionName(Param1: Type, Param2: Type) =>
    // Indented query body block
```

#### `GET` Statement

The `GET` statement is used to retrieve data from the graph.
The `GET` statement can be followed by the node or edge type to start from or it can be left empty to get all nodes and edges or followed by `NODES` to get all Nodes or by `EDGES` to get all edges.

```rust
GET User... // Get all nodes of type User, the fact user is a node is inferred from the schema
GET NODES... // Get all nodes
GET EDGES... // Get all edges
GET ... // Get all nodes and edges
```

#### Variable assignment

The `<-` operator is used to assign the result of a query to a variable. The variable name must be a valid variable name and must be followed by the `<-` operator and the query to assign the result of the query to the variable.

```rust
GET User <- User // Assign the result of the query to the variable User
    Followers <- In::Follows // Assign the result of the query to the variable Followers
    // ...
```

You can then use the variables as part of other/sub traversals or in the `RETURN` statement.

```rust
RETURN User, Followers 
```
```rust 
GET User <- User
    Followers <- In::Follows
    
RETURN User, Followers, NumberOfFollowers: COUNT(Followers)
```

## Node and Edge Traversal

#### Fetching by ID

When you want to fetch a specific node or edge by its ID, you can pass the ID as a UUID as a parameter to the node or edge type. The ID passed must be a valid UUID string.

```rust
GET User(userID) // Get a user by their ID
```

---

#### `In` Traversal

The `In` keyword is used to traverse from the current node to the node at the end of incoming edges.

```rust
GET User::In // Get all nodes that have an incoming edge to a user
```

You can specify the type of edge to traverse by using the edge type after the `In` keyword.

```rust
GET User::In::Follows // Get all nodes that have an incoming Follows edge the specified user
```

---

#### `Out` Traversal

The `Out` keyword is used to traverse from the current node to the node at the end of outgoing edges.

```rust
GET User::Out // Get all nodes that the specified user has an outgoing edge to
```

You can specify the type of edge to traverse by using the edge type after the `Out` keyword.

```rust
GET User::Out::Follows // Get all nodes that the specified user has an outgoing Follows edge to
```

---

#### `InE` Traversal

The `InEdge` keyword is used to traverse from the current node to the incoming edge.

```rust
GET User::InE // Get all incoming edges to the specified user
```

You can specify the type of node to traverse by using the node type after the `InEdge` keyword.

```rust
GET User::InE::Follows // Get all incoming Follows edges to the specified user
```

---

#### `OutE` Traversal

The `OutEdge` keyword is used to traverse from the current node to the outgoing edge.

```rust
GET User::OutE // Get all outgoing edges from the specified user
```

You can specify the type of node to traverse by using the node type after the `OutEdge` keyword.

```rust
GET User::OutE::Follows // Get all outgoing Follows edges from the specified user
```

---

#### `WHERE` Statement

The `WHERE` statement is used to filter the results of the query. The `WHERE` statement must be followed by a condition that evaluates to a boolean value. The condition can be a simple comparison or a complex expression. If the condition evaluates to `true`, the result is included in the output, otherwise it is filtered out. The condition can use the schema fields to compare against the value returned in the where clause.

i.e:

```rust
WHERE Status::Active // Filter out followers who are not active
```

This will compare the Status field of the object being passed into the node and will check if it's type is Active. Note that `Status::Active` acts as an algebraic data type and is not a string comparison. e.g. `WHERE Status::Active` is not the same as `WHERE Status == "Active"`.

---

#### `LIMIT` Statement

The `LIMIT` statement is used to limit the number of results returned by the query. The `LIMIT` statement must be followed by a positive integer value that specifies the maximum number of results to return. If the number of results exceeds the limit, the excess results are discarded.

```rust
LIMIT 50 // Limit the number of results to 50
```

---

#### `RETURN` Statement

The `RETURN` statement is used to specify the fields to return in the output of the query. The `RETURN` statement can be followed by a comma-separated list of field names. The field names must be valid field names from the schema. The fields are returned in the order specified in the `RETURN` statement. If no fields are specified, all fields are returned. 

NOTE: The `RETURN` statement returns data as a list by default unless specified otherwise.

```rust
RETURN Username, FollowerCount, FollowingCount // Return the username, follower count, and following count of the followers
RETURN // Return all fields
```

**Returning Specific Fields:**
You can return specific fields of a node by doing the following:

```rust
QUERY GetAllUsersAndFollowers =>
    GET User <- User
        Followers <- In::Follows
    RETURN User::{Username, FollowerCount}, Followers::{Username, FollowerCount}
    /* This would return 
    [
        [
            User: {
                Username: "John", 
                FollowerCount: 10
            }, 
            Followers: [
                {
                    Username: "Jane", 
                    FollowerCount: 20
                }, 
                ...
            ]
        ]
    ]
    */
```

##### `RETURN...JSON` Statement

The `TO JSON` statement is used to convert the output of the query to a JSON object. The `TO JSON` statement must be the last statement in the query. The output of the query is converted to a JSON object with the field names as keys and the field values as values. The JSON object is returned as the result of the query.

```rust
RETURN Username, FollowerCount, FollowingCount
JSON // Convert the output of the query to a JSON object
```

##### `RETURN...NEXT` Statement

The `NEXT` statement is used to return the next result in the query. Calling this statement will return the next result in the query. If there are no more results, it will return an empty result. This statement can be used to iterate over the results of the query.
```rust
RETURN Username, FollowerCount, FollowingCount
NEXT // Return the next result in the query
```
---
