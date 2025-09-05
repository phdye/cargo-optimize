# Context Boundaries for cargo-optimize Development

## CRITICAL: Two Distinct Contexts

This project operates in TWO completely separate contexts with DIFFERENT rules:

### Context A: PRODUCT FUNCTIONALITY
**What cargo-optimize DOES for its users**
- Creates optimization scripts for end users
- Generates automated build configurations  
- Produces shell scripts and config files
- Automates build optimization processes

### Context B: DEVELOPMENT WORK
**Working ON cargo-optimize itself**
- Fixing compilation errors
- Resolving warnings
- Modifying source code
- Maintaining the project

## 🚫 NEVER CONFUSE THESE CONTEXTS

### When in Context A (Product Features):
✅ DO:
- Design automation features
- Plan script generation capabilities
- Document how cargo-optimize creates scripts
- Test the scripts that cargo-optimize generates

### When in Context B (Development):
✅ DO:
- Edit files DIRECTLY using filesystem tools
- Follow Error-Resolution-Strategy.md
- Make backups before editing
- Verify changes immediately

❌ NEVER:
- Create scripts to fix compilation errors
- Generate "fix scripts" for development work
- Write bash/shell scripts to modify source code
- Produce indirect solutions

## 📋 Context Identification Checklist

Ask yourself:
1. Am I working ON cargo-optimize code? → Context B → DIRECT ACTION
2. Am I designing what cargo-optimize produces? → Context A → Script generation is OK
3. Am I fixing errors/warnings? → Context B → EDIT FILES DIRECTLY
4. Am I implementing a feature? → Context B → EDIT FILES DIRECTLY
5. Am I documenting what users get? → Context A → Describe generated scripts

## 🎯 Clear Rules for Each Context

### Context B Rules (DEVELOPMENT):
1. **ALWAYS use direct file editing tools**
2. **NEVER create intermediate scripts**
3. **Follow the systematic approach:**
   - Read file
   - Create backup
   - Edit file
   - Verify result
4. **No shell scripts for fixing code**
5. **No "run this to fix" scripts**

### Context A Rules (PRODUCT):
1. Design automated solutions
2. Generate optimization scripts
3. Create user-facing automation
4. Document script generation

## 🔴 Red Flags (You're in the wrong context!)

If you're about to write:
- "Run this script to fix the errors" → STOP! Edit directly.
- "Here's a bash script to resolve warnings" → STOP! Edit directly.
- "Execute this to apply fixes" → STOP! Edit directly.

## ✅ Green Flags (Correct context)

You should be:
- Using `Filesystem:edit_file` to modify code
- Creating `.bkp` files before changes
- Directly applying fixes
- Reporting what you DID, not what to DO

## 📝 Example Scenarios

### Scenario 1: Compilation Errors
**Task**: Fix compilation errors in tests
**Context**: B (Development)
**Approach**: 
- ❌ WRONG: "Here's a script to fix all errors"
- ✅ RIGHT: Directly edit each file using filesystem tools

### Scenario 2: Design Optimization Feature
**Task**: Plan how cargo-optimize generates build scripts
**Context**: A (Product)
**Approach**:
- ✅ RIGHT: Design script generation logic
- ✅ RIGHT: Plan what scripts to create for users

### Scenario 3: Fix Warnings
**Task**: Resolve unused variable warnings
**Context**: B (Development)
**Approach**:
- ❌ WRONG: "Save this fix_warnings.sh script"
- ✅ RIGHT: Edit files directly, prefix variables with _

## 🛡️ Context Guard Phrases

Before any action, state clearly:
- "This is DEVELOPMENT work, I will edit files directly"
- "This is PRODUCT design, about what cargo-optimize generates"

## 📌 Remember

The fact that cargo-optimize CREATES scripts doesn't mean we USE scripts to develop it.

- Mechanics don't drive cars to fix cars
- Chefs don't eat food to cook food  
- We don't write scripts to fix script-generators

**When developing cargo-optimize: DIRECT ACTION ONLY**
