<div align="center">
  
# Remyx

Framework for building TUIs on top of Ratatui.  
It focuses on simplicity and ease of use, and it is inspired by Iced.

<img width="600" height="300" alt="html_picker" src="https://github.com/user-attachments/assets/8b23549f-7785-4c52-9b80-cf6b499e5dbe" />

</div>

## Overview

Remyx follows the Elm architecture and is fully compatible with Ratatui widgets.

The framework lets you focus on describing the UI in a declarative way and implementing your application logic, handling behind the scenes the event loop, async tasks, and terminal interactions automatically.

The main trait to implement is `Application`, where you define the `view` and `update` logic.

You can also use the `subscription()` method to define async stream sources that produce your custom `Message` type. These messages are automatically routed to the update logic.

Async tasks can be spawned during initialization or inside the update function by providing a future that returns a message.

## Widget element's 

If you want to build custom widgets and / or manage their state internally (such as navigation, multi-step flows, mouse interactions, ...), you can define your own `Element` type.

`Element` is the rendering abstraction used by Remyx. Each element can contain its own internal state, so your application does not need to manage widget state directly.

The behavior and rendering of an element can change through state updates inside the element update hook.

## Current Support

At the moment, `Element` is implemented for all stateless Ratatui widgets and for `List` (renamed to `PickList`). This also serves as an example of how internal state can be hidden inside the element itself.

## Examples

You can find usage examples in the `/examples` folder.
