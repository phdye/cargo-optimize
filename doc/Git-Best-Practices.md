# Git Best Practices for cargo-optimize Development

## CRITICAL: Preserving Version History

### File Operations - ALWAYS Use Git Commands

**Moving Files**
```bash
# WRONG - Loses version history
mv src/old_file.rs src/new_location/old_file.rs

# CORRECT - Preserves version history
git mv src/old_file.rs src/new_location/old_file.rs
```

**Renaming Files**
```bash
# WRONG - Loses version history
mv src/old_name.rs src/new_name.rs

# CORRECT - Preserves version history
git mv src/old_name.rs src/new_name.rs
```

**Deleting Files**
```bash
# WRONG - Just removes the file
rm src/obsolete.rs

# CORRECT - Properly stages deletion
git rm src/obsolete.rs
```

## Recovery from Mistakes

### If You Accidentally Moved Files Without git mv

1. **Restore the deleted files**
```bash
git restore src/deleted_file.rs
# Or restore multiple files
git restore src/*.rs
```

2. **Remove the incorrectly created directory**
```bash
rm -rf src/incorrectly_created_dir/
```

3. **Use git mv properly**
```bash
git mv src/file.rs src/new_location/file.rs
```

### Checking File History

**View history of a moved file**
```bash
# Git will follow renames if you use --follow
git log --follow src/new_location/file.rs
```

**See what was renamed in a commit**
```bash
git show --name-status
```

## Best Practices

1. **Always use git commands for file operations** when the files are already tracked in git
2. **Check git status before committing** to ensure moves are recognized as renames
3. **Use git mv even for batch operations**:
   ```bash
   # Moving multiple files
   for file in src/*.rs; do
     git mv "$file" "src/old_modules/$(basename $file)"
   done
   ```

4. **PowerShell/Windows equivalent**:
   ```powershell
   # In PowerShell (as used in this project)
   cd C:\-\cygwin\root\home\phdyex\my-repos\cargo-optimize
   git mv src/analyzer.rs src/old_modules/analyzer.rs
   git mv src/cache.rs src/old_modules/cache.rs
   # ... etc
   ```

## Why This Matters

- **Version history** is crucial for understanding code evolution
- **Blame/annotation** helps identify when and why changes were made  
- **Following renames** allows tracking bugs across file moves
- **PR reviews** are clearer when git recognizes renames vs delete+add

## Quick Reference

| Operation | Wrong Way | Right Way |
|-----------|-----------|-----------|
| Move file | `mv old new` | `git mv old new` |
| Rename file | `mv old.rs new.rs` | `git mv old.rs new.rs` |
| Delete file | `rm file.rs` | `git rm file.rs` |
| Move directory | `mv old_dir/ new_dir/` | `git mv old_dir/ new_dir/` |

## Platform Note

This project is developed on Windows with Cygwin. Both of these work:
- Git Bash / Cygwin: `git mv src/file.rs dest/file.rs`
- PowerShell: `git mv src\file.rs dest\file.rs`

Git handles path separators correctly on all platforms.

---

**Remember**: If git status shows "deleted" and "new file" instead of "renamed", you've lost the history connection. Use the recovery steps above to fix it!
