# Customization

Starlight is build using **GTK** styling is done using **CSS**

Refer to below css resources to learn basic styling:

- [CSS tutorial](https://www.w3schools.com/css/)
- Gtk4
  - [GTK4 CSS Overview wiki](https://docs.gtk.org/gtk4/css-overview.html)
  - [GTK4 CSS Properties Overview wiki](https://docs.gtk.org/gtk4/css-properties.html)

> [!WARNING] GTK is not the web
>
> While most features are implemented in GTK, you can't assume anything that
> works on the web will work with GTK. Refer to the **GTK** **CSS** docs to see what is
> supported.

## Custom css

Create a `starlight.css` in `~/.config/starlight/`

Reference from default css for `starlight`

:::code-group

``` css
/* main window */
window {
    border-radius: 16px;
    color: #f0f0f0;
    font-family: "SF Pro", "Inter", "Roboto", sans-serif;
    font-size: 14px;
    opacity: 0.9;
}

/* main content box */
.content {
    border-radius: 16px;
    padding: 12px;
}

/* search Entry */
entry.search-entry {
    color: #ffffff;
    border-radius: 12px;
    padding: 6px 10px;
}
entry.search-entry:focus {
    border-color: #007aff;
    box-shadow: 0 0 0 2px rgba(0, 122, 255, 0.4);
}

/* loading spinner and label */
.dim-label {
    color: #aaaaaa;
    font-style: italic;
}

/* listbox for applications  */
.apps-list {
    background-color: transparent;
    border: none;
}

/* apps row*/
row.card {
    background-color: rgba(50, 50, 50, 0.5);
    border-radius: 8px;
    margin-bottom: 6px;
    transition: background-color 0.2s ease, transform 0.1s ease;
}
row.card:hover {
    background-color: rgba(80, 80, 80, 0.7);
    transform: scale(1.02);
}

/* app title */
label.title {
    font-weight: 600;
    font-size: 15px;
    color: #f5f5f7;
}

.scrolled-window {
    background-color: transparent;
    border: none;
}

```

> [!WARNING] Keeping the starlight.css empty may cause styling issue
