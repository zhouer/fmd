#!/usr/bin/env bash
#
# mdq — Markdown Query Tool
# A find-like tool for Markdown files with metadata awareness.
# Enumerates and filters Markdown files by tags, title, and filename.
# Outputs filenames to stdout (newline or NUL-delimited).
#
# © 2025 Enjan Chou — MIT License

set -euo pipefail

# ---------- Default settings ----------
HEAD_LINES=10         # number of lines to scan for metadata
READ_MODE="nl"        # nl | nul
GLOB="*.md"
MAX_DEPTH=""          # empty = recursive, "1" = non-recursive
IGNORE_CASE=false     # case-insensitive filename matching

# ---------- Helper functions ----------
die(){ echo "mdq: $*" >&2; exit 1; }
is_tty_stdin(){ [ -t 0 ]; }
emit(){ [[ "$READ_MODE" = "nul" ]] && printf '%s\0' "$1" || printf '%s\n' "$1"; }

usage(){
cat <<'EOF'
mdq — find for Markdown files with metadata awareness

Usage:
  mdq [-0] [-i] [-d DEPTH] [-t TAG]... [-T TITLE]... [-n NAME]... [-f FIELD:PAT]... [DIR...]

Options:
  -0                 Use NUL-delimited output (safe for xargs -0)
  -i, --ignore-case  Case-insensitive filename matching (tags/title/fields already case-insensitive)
  -d, --depth N      Limit search depth (1=current dir only, default: unlimited/recursive)
  -t, --tag TAG      Filter by tag (searches "tags:" metadata, case-insensitive)
  -T, --title PAT    Filter by title (searches "title:" or "# heading", case-insensitive)
  -n, --name PAT     Filter by filename (regex pattern, case-sensitive by default, use -i for case-insensitive)
  -f, --field F:P    Filter by frontmatter field (format: "field:pattern", case-insensitive)
  --glob GLOB        File pattern to match (default: *.md)
  --head N           Lines to scan for metadata (default: 10)
  -h, --help         Show this message

YAML Frontmatter Support:
  mdq now supports both simple inline metadata and full YAML frontmatter.

  Simple format:    # My Document Title
                    tags: #Linux #macOS

  YAML format:      ---
                    title: My Document Title
                    tags: [Linux, macOS]
                    author: John Doe
                    date: 2025-11-04
                    ---

Filter Logic:
  Same type (multiple -t, -T, -n, or -f): OR logic
    -t A -t B         → files with tag A OR tag B
    -f author:John -f author:Jane  → author=John OR author=Jane
  Cross type (-t AND -T AND -n AND -f): AND logic
    -t A -T X         → files with tag A AND title matching X
    -t A -f status:draft  → tag=A AND status=draft
    -t A -T X -n Y -f author:John  → tag A AND title X AND filename Y AND author John

Examples:
  mdq                                   # list all *.md recursively from current directory
  mdq -d 1                              # list only in current directory (non-recursive)
  mdq -t Linux -t MacOS                 # tag=Linux OR tag=MacOS (recursive)
  mdq -n "2025"                         # filename contains "2025" (recursive)
  mdq -t Linux -T "Setup"               # tag=Linux AND title="Setup"
  mdq -f "author:John"                  # author field contains "John"
  mdq -f "status:draft"                 # status field is "draft"
  mdq -f "date:2025-11"                 # date contains "2025-11"
  mdq -t Work -f "author:John"          # tag=Work AND author=John
  mdq -t Finance | xargs grep -l 'Apple'        # combine with grep
  mdq -d 1 -t Linux | xargs -I {} mv {} ./topics/     # non-recursive move
EOF
}

# ---------- Metadata helpers ----------

# Extract YAML frontmatter from a file (content between --- delimiters)
extract_frontmatter(){
  local file="$1"
  awk '
    BEGIN { in_fm = 0; started = 0 }
    /^---[[:space:]]*$/ {
      if (!started) { started = 1; in_fm = 1; next }
      else if (in_fm) { exit }
    }
    in_fm { print }
  ' "$file" 2>/dev/null
}

# Get metadata content (YAML frontmatter if exists, else first N lines)
get_metadata(){
  local file="$1"
  local fm
  fm=$(extract_frontmatter "$file")

  if [[ -n "$fm" ]]; then
    echo "$fm"
  else
    head -n "$HEAD_LINES" -- "$file" 2>/dev/null
  fi
}

# Check if file has field matching pattern (supports both YAML and simple format)
# $1 = file, $2 = field name, $3 = pattern
has_field(){
  local file="$1" field="$2" pattern="$3"
  local metadata
  metadata=$(get_metadata "$file")

  # Match "field: value" or YAML array "- item" under field
  # Use tolower() for case-insensitive matching (mawk compatible)
  echo "$metadata" | awk -v field="$field" -v pat="$pattern" '
    BEGIN { in_array=0; found=0; pat_lower=tolower(pat) }
    tolower($0) ~ "^" tolower(field) ":" {
      # Inline format: "field: value" or "field: [a, b]"
      if (tolower($0) ~ pat_lower) { found=1; exit }
      # Check if next lines are array items
      in_array = 1
      next
    }
    in_array && /^[[:space:]]*-/ {
      if (tolower($0) ~ pat_lower) { found=1; exit }
      next
    }
    in_array && /^[[:alpha:]]/ { in_array = 0 }
    END { exit !found }
  '
}

has_tag(){
  # $1 = file, $2 = tag pattern (with or without # prefix)
  # Supports YAML: tags: [A, B] or tags:\n  - A
  # Supports simple: tags: #A #B
  local file="$1" pattern="$2"

  # Try with the pattern as-is (works for YAML and simple without #)
  if has_field "$file" "tags" "$pattern"; then
    return 0
  fi

  # If pattern starts with #, try without it (for YAML compatibility)
  if [[ "$pattern" =~ ^# ]]; then
    local pattern_no_hash="${pattern#\#}"
    if has_field "$file" "tags" "$pattern_no_hash"; then
      return 0
    fi
  fi

  return 1
}

has_title(){
  # $1 = file, $2 = title pattern (case-insensitive)
  local file="$1" pattern="$2"

  # Try YAML title field first
  if has_field "$file" "title" "$pattern"; then
    return 0
  fi

  # Fall back to markdown heading
  head -n "$HEAD_LINES" -- "$file" 2>/dev/null | grep -Eiq "^# .*$pattern"
}

has_name(){
  # $1 = file, $2 = filename pattern
  # Use bash parameter expansion instead of basename (avoids fork/exec)
  local filename="${1##*/}"

  if [[ "$IGNORE_CASE" = true ]]; then
    # Case-insensitive: convert both to lowercase for comparison
    shopt -s nocasematch
    [[ "$filename" =~ $2 ]]
    local result=$?
    shopt -u nocasematch
    return $result
  else
    # Case-sensitive (default)
    [[ "$filename" =~ $2 ]]
  fi
}

# ---------- Input enumeration ----------
enum_paths(){
  # Enumerate files from arguments or current directory
  if [[ $# -eq 0 ]]; then set -- "."; fi

  # Build find command with optional maxdepth
  local find_args=()
  for dir in "$@"; do
    if [[ -d "$dir" ]]; then
      find_args+=("$dir")
    fi
  done

  # If no valid directories, return
  [[ ${#find_args[@]} -eq 0 ]] && return

  # Add maxdepth if specified
  if [[ -n "$MAX_DEPTH" ]]; then
    if [[ "$READ_MODE" = "nul" ]]; then
      find "${find_args[@]}" -maxdepth "$MAX_DEPTH" -type f -name "$GLOB" -print0
    else
      find "${find_args[@]}" -maxdepth "$MAX_DEPTH" -type f -name "$GLOB" -print
    fi
  else
    # Recursive (no maxdepth limit)
    if [[ "$READ_MODE" = "nul" ]]; then
      find "${find_args[@]}" -type f -name "$GLOB" -print0
    else
      find "${find_args[@]}" -type f -name "$GLOB" -print
    fi
  fi
}

# ---------- Main logic ----------
main(){
  # Parse tags, titles, names, fields, and options
  local -a TAGS=()
  local -a TITLES=()
  local -a NAMES=()
  local -a FIELDS=()
  local -a DIRS=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -d|--depth)
        shift
        [[ $# -ge 1 ]] || die "-d requires a depth argument"
        MAX_DEPTH="$1"
        shift
        ;;
      -t|--tag)
        shift
        [[ $# -ge 1 ]] || die "-t requires a tag argument"
        TAGS+=("$1")
        shift
        ;;
      -T|--title)
        shift
        [[ $# -ge 1 ]] || die "-T requires a title pattern argument"
        TITLES+=("$1")
        shift
        ;;
      -n|--name)
        shift
        [[ $# -ge 1 ]] || die "-n requires a filename pattern argument"
        NAMES+=("$1")
        shift
        ;;
      -f|--field)
        shift
        [[ $# -ge 1 ]] || die "-f requires a field:pattern argument"
        FIELDS+=("$1")
        shift
        ;;
      -*) die "unknown option: $1";;
      *) DIRS+=("$1"); shift;;
    esac
  done

  # If no directories specified, use captured directories or default to current
  if ((${#DIRS[@]}==0)); then
    DIRS=(".")
  fi

  # If no filters specified, just list all files
  if ((${#TAGS[@]}==0 && ${#TITLES[@]}==0 && ${#NAMES[@]}==0 && ${#FIELDS[@]}==0)); then
    enum_paths "${DIRS[@]}"
  else
    # Filter logic:
    # - Same type (multiple -t, -T, -n, or -f): OR
    # - Cross type (-t AND -T AND -n AND -f): AND
    enum_paths "${DIRS[@]}" | {
      if [[ "$READ_MODE" = "nul" ]]; then
        while IFS= read -r -d '' f; do
          [[ -f "$f" ]] || continue

          # Check filenames FIRST (fast, no file I/O needed)
          if ((${#NAMES[@]}>0)); then
            local name_matched=false
            for name in "${NAMES[@]}"; do
              if has_name "$f" "$name"; then name_matched=true; break; fi
            done
            [[ "$name_matched" = false ]] && continue
          fi

          # Check tags (requires file I/O)
          if ((${#TAGS[@]}>0)); then
            local tag_matched=false
            for t in "${TAGS[@]}"; do
              if has_tag "$f" "#$t"; then tag_matched=true; break; fi
            done
            [[ "$tag_matched" = false ]] && continue
          fi

          # Check titles (requires file I/O)
          if ((${#TITLES[@]}>0)); then
            local title_matched=false
            for title in "${TITLES[@]}"; do
              if has_title "$f" "$title"; then title_matched=true; break; fi
            done
            [[ "$title_matched" = false ]] && continue
          fi

          # Check frontmatter fields (requires file I/O)
          if ((${#FIELDS[@]}>0)); then
            local field_matched=false
            for fspec in "${FIELDS[@]}"; do
              # Parse field:pattern format
              if [[ "$fspec" =~ ^([^:]+):(.+)$ ]]; then
                local fname="${BASH_REMATCH[1]}"
                local fpat="${BASH_REMATCH[2]}"
                if has_field "$f" "$fname" "$fpat"; then field_matched=true; break; fi
              fi
            done
            [[ "$field_matched" = false ]] && continue
          fi

          # All filters passed
          emit "$f"
        done
      else
        while IFS= read -r f; do
          [[ -f "$f" ]] || continue

          # Check filenames FIRST (fast, no file I/O needed)
          if ((${#NAMES[@]}>0)); then
            local name_matched=false
            for name in "${NAMES[@]}"; do
              if has_name "$f" "$name"; then name_matched=true; break; fi
            done
            [[ "$name_matched" = false ]] && continue
          fi

          # Check tags (requires file I/O)
          if ((${#TAGS[@]}>0)); then
            local tag_matched=false
            for t in "${TAGS[@]}"; do
              if has_tag "$f" "#$t"; then tag_matched=true; break; fi
            done
            [[ "$tag_matched" = false ]] && continue
          fi

          # Check titles (requires file I/O)
          if ((${#TITLES[@]}>0)); then
            local title_matched=false
            for title in "${TITLES[@]}"; do
              if has_title "$f" "$title"; then title_matched=true; break; fi
            done
            [[ "$title_matched" = false ]] && continue
          fi

          # Check frontmatter fields (requires file I/O)
          if ((${#FIELDS[@]}>0)); then
            local field_matched=false
            for fspec in "${FIELDS[@]}"; do
              # Parse field:pattern format
              if [[ "$fspec" =~ ^([^:]+):(.+)$ ]]; then
                local fname="${BASH_REMATCH[1]}"
                local fpat="${BASH_REMATCH[2]}"
                if has_field "$f" "$fname" "$fpat"; then field_matched=true; break; fi
              fi
            done
            [[ "$field_matched" = false ]] && continue
          fi

          # All filters passed
          emit "$f"
        done
      fi
    }
  fi
}

# ---------- Argument parsing ----------
while [[ $# -gt 0 ]]; do
  case "$1" in
    -0) READ_MODE="nul"; shift;;
    -i|--ignore-case) IGNORE_CASE=true; shift;;
    --glob) shift; GLOB="$1"; shift;;
    --head) shift; HEAD_LINES="$1"; shift;;
    -h|--help) usage; exit 0;;
    *) break;;
  esac
done

# ---------- Run main ----------
main "$@"
