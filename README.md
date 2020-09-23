# Crochet, an exploration into reactive UI

This repo contains a prototype for exploration into a possible next-generation reactive UI architecture for [Druid]. For background, see the blog post [Towards a unified theory of reactive UI]. It is a fusion of ideas from many sources, including [imgui], [Jetpack Compose], [Moxie], [Makepad], [Conrod], and others.

The code is not too complex, and I hope people will find reading it (or at least skimming) to be rewarding. Many (but not yet all) of the main types have docstring, so running `cargo doc --open` is not a bad way to navigate the code. (TODO item: is it reasonably easy to make a Github Action that runs cargo doc, so it doesn't have to be done locally?)

## The Crochet architecture

The Crochet architecture centers around a "view tree" (probably also referred to as the "crochet tree" in discussions). A node in this tree is the *description* of a render object (widget). The central concept of Crochet is this: running your app logic produces a *mutation* of the view tree. A tree mutation is a delta that describes the new state of the view tree as a sparse set of changes from the old state.

The main interface between the app logic and the toolkit is the Crochet context (usually referred to as "cx"). The app declaratively expresses the new state of the view tree by method calls on the crochet context.

As in Jetpack Compose and Moxie, tree nodes have a *stable identity,* determined by keys. If a run of the app logic specifies a node with a new key that didn't exist in the old tree (as will always be the case on first run), a node is inserted into the tree. In reverse, if a key is not present in a run of the app logic, the node is deleted. Otherwise, the node is retained with a stable identity; its state is not modified. The attributes and children can be updated in such cases.

By default, the key is the *caller* of the method producing the node, along with the sequence number of nodes produced with that caller. As in Moxie, Rust's new `#[track_caller]` feature is used to provide a unique call-site identity. (Jetpack Compose also uses unique identities, but uses a Kotlin compiler plugin to generate them.)

After the app logic runs, producing a tree mutation, it is applied to the render object (widget tree). Basically, this brings the render object tree in sync with the view tree. For newly inserted nodes, a new widget is created, based on the description in the view.

The crochet tree also contains *memoization state,* using a mechanism similar to Moxie and Jetpack Compose. This will also be familiar to users of React hooks, though the [Rules of Hooks](https://reactjs.org/docs/hooks-rules.html) are considerably relaxed in Crochet vs React because the track_caller mechanism lets the context keep its place much more accurately than simply relying on the sequence number in its corresponding array.

One of the main ways that Crochet differs from Jetpack Compose is that the context is also used to access *actions* from events, for example button clicks. Each node in the view tree (conceptually) has an associated *action queue*. Typically, the app logic retrieves actions from this action queue in the same method call it uses to emit the item. The pattern `if button(...) { ... }` will be familiar to imgui users.

### Skipping

A major performance feature is *skipping.* Without skipping, the app logic emits the entire view tree on every run, which is convenient but wasteful if most of the tree hasn't changed.

One of the main skipping methods is `if_changed`, which takes a data argument. If the data has changed (or if the node is being inserted for the first time), then it runs the closure that represents its body. Otherwise, it skips it, effectively copying the corresponding subtree (at basically no cost) from the previous state of the tree. This mechanism is similar to that provided in Jetpack Compose, and is a familiar caching pattern in imgui as well.

One advantage to accessing the action queues through the Crochet context (as opposed to having separate callbacks for actions, as in Jetpack Compose), is that the skipping logic can be sensitive to actions as well. The `if_changed` method also checks for any nonempty action queues in the subtree, and traverses into the subtree if so.

A hypothesis to be tested in this exploration is that skipping based on explicit (but hopefully low-friction) app logic and the action queues provides efficient dispatching of events, and overall a sparse, incremental performance profile. One reason to be hopeful is that ordinary performance engineering techniques such as profiling and tracing should be effective in guiding the developer where to apply more careful skipping logic, as the control flow remains very simple.

### Low level tree mutation

While a declarative approach to creating view nodes is appropriate for random app logic, it is less appealing for structured collections, especially as the scale goes up. For these use cases, I envision opening access to a lower level API, which would require some tracking of the old state of the tree on the application side. Even so, if this logic is encapsulated in a component, it should still be relatively easy to use.

The low level tree mutation API would be expected to support "skip n", "delete n", and versions of update/insert that make that distinction explicitly rather than inferring it from keys. I *believe* in such cases we don't even need to supply unique id's for items, as the responsibility for tracking widget identity falls entirely on the user of the low level API.

To be useful, this low level API also needs to identify which children have nonempty action queues. That way, for example, a list view would be able to dispatch a button click within one of its items to the app code for that item.

### Open questions

There are many. One is whether to support reordering of children within a node. I think the answer is yes, but in the current code, if the tree is A, B and the next run of the app logic produces B, A, then it will delete the A and insert a new instance after the B.

## Contributing

This repo is for experimentation and exploration. It uses an [optimistic merging] policy; feel free to make any changes you feel contribute to the goal of learning something. Commit access will be freely given. The project follows the [Rust code of conduct].

The following items are of particular interest:

* Adapting more widgets to the `AnyWidget` enum (we're trying to avoid patching Druid, for project coordination reasons).
* Working out efficient data structures and algorithms for the tree mutation primitives.
* Experimenting with larger scale collections such as lists and a tree view.
* Render objects other than widgets, for example tree view items.
* Deeper exploration of async integration.

But overall the goal is to gather evidence for whether this architecture is viable.

## License

All files in this repo are released under an Apache 2.0 license. Some code has been cut and pasted from Druid, therefore carries a copyright of "The Druid Authors". See the [Druid repo] for the definitive authors list.

[Druid repo]: https://github.com/linebender/druid
[optimistic merging]: http://hintjens.com/blog:106
[Rust code of conduct]: https://www.rust-lang.org/policies/code-of-conduct
[Towards a unified theory of reactive UI]: https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html
[Jetpack Compose]: https://medium.com/androiddevelopers/under-the-hood-of-jetpack-compose-part-2-of-2-37b2c20c6cdd
[imgui]: https://github.com/ocornut/imgui
[Moxie]: https://moxie.rs/
[Makepad]: https://github.com/makepad/makepad
[Conrod]: https://github.com/PistonDevelopers/conrod
[Composer]: https://developer.android.com/reference/kotlin/androidx/compose/runtime/Composer
[Rules of Hooks]: https://reactjs.org/docs/hooks-rules.html
