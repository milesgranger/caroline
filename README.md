caroline
---


[![CircleCI](https://circleci.com/gh/milesgranger/caroline/tree/master.svg?style=svg)](https://circleci.com/gh/milesgranger/caroline/tree/master)

---

**Work in progress / hobby project**

Create AWS Cloud formation templates using Rust. Code is completely generated
from the CloudFormationResourceSpecification files.

---

Overall status:

- [X] Implement the generation of "PropertyTypes"
- [X] Build structs from AWS CloudFormationResourceSpecificiation(s)
- [X] Implement `::new(..)` methods for them.
- [X] Use struct parameter types from other types specified in the specification file.
- [ ] Implement the generation of "ResourceTypes"
- [ ] Implement a `CloudFormation` obj of sorts, which handles dependency resolutions
