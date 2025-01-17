# Query KDL - Specifications

This document describe how to query [KDL](https://kdl.dev) format using this crate. 

## General descirption

This specification is inspired from [XPath](https://www.w3.org/TR/1999/REC-xpath-19991116/) and KDL syntax it to make selection simple and intuitive, especially if you are familiar with XPath.

## Node selection

Node selection is pretty similar to what XPath does.

| **symbol** | **description** |
|:--:|:---|
| `<name>` | Selects named nodes from the current node |
| `/` | At the beginning of the query, selects the root node (the document). This is also the separator between nodes |
| `//` | Selects each node of the document. |
| `*` | Selects each node of the current node. |
| `.` | Selects current node |
| `..` | Selects parent's node (from the current node) |
