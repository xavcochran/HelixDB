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
g.V()
  .hasLabel('user')
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
MATCH (follower:user)-[:follows]->(user:user)
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
    ?user a :user .
}
LIMIT 50
```

### HelixQL Example
    
```rust
use Schema::{Users};                                // import the schema

GET Users::Followers DISTINCT -> User               // get all the followers of a user returning the user
    WHERE Status::Active                            // filter out followers who are not active
    LIMIT 50                                        // limit the number of followers to 50 
    RETURN Username, FollowerCount, FollowingCount  // return the list of nodes as the following fields
```