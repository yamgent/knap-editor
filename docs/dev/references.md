# References

We stand on the shoulders of giants. Most of the editor's implementation is 
inspired by the following crates (of which some are used directly),
reading materials, codebases, etc.

## Editor Foundations

- Reading materials:
    - [Hecto Editor tutorial](https://flenker.blog/hecto/)
- Crates:
    - `unicode-segmentation`
    - `unicode-width`

## Text Buffer

- Reading materials:
    - [vscode Piece Table](https://code.visualstudio.com/blogs/2018/03/23/text-buffer-reimplementation)
        - Ultimately we did not use a piece table, but the content helped us understand the problem space.
- Crates:
    - `ropey`
        - Also used within helix.

## UI (Layout, Rendering, Events, etc.)

- Reading materials:
    - [Why UI layout calculations are slow](https://mortoray.com/why_ui_layout_calculations_are_slow/)
    - [Writing a UI engine](https://mortoray.com/topics/writing-a-ui-engine/)
- Crates:
    - `linebender::parley`
        - We did not use this crate, but it inspired our textbox implementation.
