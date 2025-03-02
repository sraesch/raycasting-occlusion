# Occlusion Detection with Ray Casting

## Overview

This is a tiny fun project to analyze methods of object occlusion testing in a 3D scene using ray casting.
A 3D scene is testing for object occlusion by casting rays from the camera to the objects in the scene.
The scene can easily be too large to have it fully in-memory or transmitted over the network.
Therefore, the algorithms should be able to handle large scenes with a low memory footprint.
We assume a preprocessing can happen to optimize the scene for the occlusion detection that may process the full scene and can consume substantial time and memory.

Thus, the general idea is to have a costly preprocessing step that optimizes the scene for a lightweight and fast occlusion detection.

See [Changelog](CHANGELOG.md) for the latest changes.

### Input

The input is a 3D scene consisting of an array of tessellated meshes and an array of objects which are instances of the meshes with unique transformations.
A scene has the following in-memory structure:

| Property | Data Type | Description |
| --- | --- | --- |
| Meshes | Mesh[] | List of meshes |
| Objects | Object[] | List of objects |

An object as the following in-memory structure:

| Property | Data Type | Description |
| --- | --- | --- |
| Mesh Index | uint32 | Index of the mesh |
| Transformation | float32[12] | Transformation matrix of the object |

The object id is the index of the object in the array.

A mesh is a collection of triangles and vertices consisting of 3D coordinates only and looks like:

| Property | Data Type | Description |
| --- | --- | --- |
| Vertices | float32[] | List of 3D coordinates of the vertices |
| Triangles | uint32[] | List of indices of the vertices forming the triangles |

### Output
The output is a sorted list of the objects in descending order of visibility. The visibility of an object is determined by how much of the final rendered image is occluded by it.
For example, 1.0 means the screen is fully occluded by the object, 0.0 means the object is fully visible.
If an object is for example hidden behind a wall, it will have a visibility of 0.0.
The output has the following in-memory structure:

| Property | Data Type | Description |
| --- | --- | --- |
| (Object Id, Visibility) | (uint32, float32) | Index of the object and visibility |