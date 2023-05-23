Currently has two commands

- `cdiff /path/to/source-of-truth.html /path/to/version-to-update.html`
    - Compares the text content of the two files and uses `vimdiff` to display the diff (see below for `nvim` config).
- `footnotes /path/to/current-version.html`
    - This goes through and updates footnote reference numbering and footnote detail ordering to be consistent.
    - Lets you drop in footnotes without needing to manually update their numbers.
        - For example, if I have footnotes 1, 2, 3, 4, 5 and then I need to insert a new one between 2 and 3, I'd have to then update 3 to be 4, 4 to be 5, etc. This will do it for you.
    - Intended usage is from within `nvim`, running `:%!rtool footnotes %`, which passes the current file path and replaces the buffer with the command's output.

## `cdiff` config

This requires that you configure `nvim` to force a line-by-line diff:

```
" In `init.vim` or equivalent

" Configure vimdiff
" to force line-by-line comparison,
" instead of trying to figure out
" what lines should go together.
set diffexpr=LineDiff()
function LineDiff()
   let opt = ""
   if &diffopt =~ "icase"
     let opt = opt .. "-i "
   endif
   if &diffopt =~ "iwhite"
     let opt = opt .. "-b "
   endif
   silent execute "!diff <(nl -ba " .. v:fname_in .. ") <(nl -ba " .. v:fname_new .. ") > " .. v:fname_out
   redraw!
endfunction
set diffopt+=followwrap " Preserve line-wrapping settings when using vimdiff
```

Otherwise it will try to guess which lines match up, which is error-prone.
