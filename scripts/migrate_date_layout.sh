#!/usr/bin/env bash
#
# migrate_date_layout.sh
# ----------------------
# One-off migration for Orderly's sort_by_date layout change:
#   OLD: <sorted_root>/YYYY/MMM/<file>       (e.g. Photos/2024/Mar/IMG.jpg)
#   NEW: <sorted_root>/YYYY/MM - MMM/<file>  (e.g. Photos/2024/03 - Mar/IMG.jpg)
#
# The new MM-prefix makes Finder sort month folders chronologically.
#
# Sources scanned (for each root pair below):
#   1. <sorted_root>/YYYY/MMM/**            — current rule output, old pattern
#   2. <sorted_root>/YYYY/MM - MMM/**       — already migrated (idempotent pass)
#   3. <root>/YYYY/MMM/**                   — loose legacy year folders at root
#   4. <root>/YYYY/MM - MMM/**              — loose but already new format
#
# Collision policy (per file):
#   * src == dst                      -> skip silently (idempotent)
#   * dst missing                     -> mkdir -p parent && mv -n
#   * dst exists, byte-identical      -> src moved to ~/.Trash (recoverable)
#   * dst exists, differs             -> dst renamed to "<name>.migrated-NN<ext>"
#
# After moves, rmdir walks up the chain and removes any now-empty
# YYYY/MMM or YYYY/MM - MMM or YYYY directories. Non-empty dirs are kept.
#
# Safety:
#   * Dry-run is the default. --apply is required to touch disk.
#   * Only directories whose basename matches strict regexes are considered.
#     Screenshots/, Wallpapers/, CleanShot/, WhatsApp/, etc. are never touched.
#   * Unrecognized month names produce a WARN line and are skipped.
#
# Usage:
#   scripts/migrate_date_layout.sh                # dry-run (default)
#   scripts/migrate_date_layout.sh --dry-run      # explicit dry-run
#   scripts/migrate_date_layout.sh --apply        # actually move files
#   scripts/migrate_date_layout.sh --verbose      # also log SKIP-SAME
#   scripts/migrate_date_layout.sh --apply --verbose

set -euo pipefail

# -------- configuration ----------------------------------------------------

# Each entry: "<root>:<sorted_root>". Files found in either path (by shape
# rules below) are consolidated under <sorted_root> using the MM - Mmm layout.
ROOT_PAIRS=(
  "$HOME/Pictures:$HOME/Pictures/Photos"
  "$HOME/Movies:$HOME/Movies/Library"
)

YEAR_REGEX='^[0-9]{4}$'
OLD_MONTH_REGEX='^[A-Z][a-z]{2}$'            # Jan, Feb, ... Dec
NEW_MONTH_REGEX='^[0-9]{2} - [A-Z][a-z]{2}$' # "03 - Mar"

# -------- flags ------------------------------------------------------------

DRY_RUN=1
VERBOSE=0

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --apply)   DRY_RUN=0 ;;
    --verbose) VERBOSE=1 ;;
    -h|--help)
      sed -n '2,40p' "$0"
      exit 0
      ;;
    *)
      echo "unknown flag: $arg" >&2
      echo "usage: $0 [--dry-run|--apply] [--verbose]" >&2
      exit 2
      ;;
  esac
done

# -------- counters ---------------------------------------------------------

moved=0
planned=0
skipped_same=0
skipped_other=0
dup_trashed=0
conflicts=0
warnings=0

# -------- helpers ----------------------------------------------------------

month_to_num() {
  case "$1" in
    Jan) echo 01 ;; Feb) echo 02 ;; Mar) echo 03 ;; Apr) echo 04 ;;
    May) echo 05 ;; Jun) echo 06 ;; Jul) echo 07 ;; Aug) echo 08 ;;
    Sep) echo 09 ;; Oct) echo 10 ;; Nov) echo 11 ;; Dec) echo 12 ;;
    *) return 1 ;;
  esac
}

# extract "Mmm" from either "Mmm" or "NN - Mmm"
month_abbrev_from_dirname() {
  local name="$1"
  if [[ "$name" =~ $OLD_MONTH_REGEX ]]; then
    echo "$name"
  elif [[ "$name" =~ $NEW_MONTH_REGEX ]]; then
    echo "${name: -3}"
  else
    return 1
  fi
}

# pick a non-colliding ".migrated-NN" name for $1
pick_conflict_name() {
  local orig="$1"
  local dir base ext stem n candidate
  dir="$(dirname "$orig")"
  base="$(basename "$orig")"
  if [[ "$base" == *.* ]]; then
    ext=".${base##*.}"
    stem="${base%.*}"
  else
    ext=""
    stem="$base"
  fi
  n=1
  while :; do
    candidate="$dir/$stem.migrated-$(printf '%02d' "$n")$ext"
    [[ ! -e "$candidate" ]] && { echo "$candidate"; return 0; }
    n=$((n + 1))
    [[ $n -gt 99 ]] && { echo "ERR_NO_CONFLICT_SLOT" >&2; return 1; }
  done
}

log()  { printf '%s\n' "$*"; }
warn() { printf 'WARN  %s\n' "$*" >&2; warnings=$((warnings + 1)); }

# Move or preview $1 -> $2. Handles collision policy.
move_file() {
  local src="$1" dst="$2"

  if [[ "$src" == "$dst" ]]; then
    skipped_same=$((skipped_same + 1))
    if [[ $VERBOSE -eq 1 ]]; then
      log "SKIP-SAME    $src"
    fi
    return 0
  fi

  if [[ -e "$dst" ]]; then
    if cmp -s "$src" "$dst"; then
      dup_trashed=$((dup_trashed + 1))
      if [[ $DRY_RUN -eq 1 ]]; then
        log "PLANNED-DUP  $src  -> <trash> (identical to existing $dst)"
      else
        mv "$src" "$HOME/.Trash/" 2>/dev/null || {
          warn "could not trash $src (does $HOME/.Trash exist?)"
          return 0
        }
        log "DUP-TRASHED  $src  -> <trash>"
      fi
      return 0
    fi

    # Byte-different -> rename the incoming file.
    local new_dst
    new_dst="$(pick_conflict_name "$dst")" || {
      warn "no free .migrated-NN slot for $dst"
      return 0
    }
    conflicts=$((conflicts + 1))
    if [[ $DRY_RUN -eq 1 ]]; then
      planned=$((planned + 1))
      log "PLANNED-CONF $src  -> $new_dst"
    else
      mkdir -p "$(dirname "$new_dst")"
      mv -n "$src" "$new_dst"
      moved=$((moved + 1))
      log "MOVED-CONF   $src  -> $new_dst"
    fi
    return 0
  fi

  if [[ $DRY_RUN -eq 1 ]]; then
    planned=$((planned + 1))
    log "PLANNED      $src  -> $dst"
  else
    mkdir -p "$(dirname "$dst")"
    mv -n "$src" "$dst"
    moved=$((moved + 1))
    log "MOVED        $src  -> $dst"
  fi
}

# -------- scan -------------------------------------------------------------

# For one root-pair, find files living under year/month shapes and consolidate
# them under <sorted_root>/YYYY/MM - Mmm/<basename>.
process_root_pair() {
  local root="$1" sorted="$2"

  [[ -d "$root" ]] || { log "INFO  skip root (missing): $root"; return 0; }

  # Candidate parents are:
  #   <sorted>/YYYY/<month-dir>    if sorted exists
  #   <root>/YYYY/<month-dir>      for loose legacy
  #
  # We only descend exactly 2 levels, to avoid touching random nested folders.
  local base
  for base in "$sorted" "$root"; do
    [[ -d "$base" ]] || continue

    local year_dir year_name month_dir month_name month_abbrev month_num
    # Iterate YYYY year directories.
    while IFS= read -r -d '' year_dir; do
      year_name="$(basename "$year_dir")"
      [[ "$year_name" =~ $YEAR_REGEX ]] || continue

      while IFS= read -r -d '' month_dir; do
        month_name="$(basename "$month_dir")"
        if ! month_abbrev="$(month_abbrev_from_dirname "$month_name")"; then
          warn "unrecognized month dir (skipped): $month_dir"
          continue
        fi
        if ! month_num="$(month_to_num "$month_abbrev")"; then
          warn "cannot map month to number (skipped): $month_dir"
          continue
        fi

        local target_dir="$sorted/$year_name/$month_num - $month_abbrev"

        # Move every regular file (1 level deep only; nested dirs here would
        # be a structural surprise and must be investigated by hand).
        local f
        while IFS= read -r -d '' f; do
          [[ -f "$f" ]] || continue
          local dst="$target_dir/$(basename "$f")"
          move_file "$f" "$dst"
        done < <(find "$month_dir" -mindepth 1 -maxdepth 1 -type f -print0 2>/dev/null)

        # Warn about nested surprises (any subdirectory inside a month folder).
        while IFS= read -r -d '' surprise; do
          warn "nested dir inside month folder (left untouched): $surprise"
          skipped_other=$((skipped_other + 1))
        done < <(find "$month_dir" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)

      done < <(find "$year_dir" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)

    done < <(find "$base" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
  done
}

# -------- cleanup ----------------------------------------------------------

# Remove any now-empty month or year directories under the root pair. `rmdir`
# intentionally fails on non-empty dirs; we ignore those failures silently.
cleanup_empty_dirs() {
  local root="$1" sorted="$2"

  # Month dirs first (deepest), then year dirs.
  for base in "$sorted" "$root"; do
    [[ -d "$base" ]] || continue
    local year_dir year_name month_dir month_name
    while IFS= read -r -d '' year_dir; do
      year_name="$(basename "$year_dir")"
      [[ "$year_name" =~ $YEAR_REGEX ]] || continue

      while IFS= read -r -d '' month_dir; do
        month_name="$(basename "$month_dir")"
        if [[ "$month_name" =~ $OLD_MONTH_REGEX || "$month_name" =~ $NEW_MONTH_REGEX ]]; then
          if [[ $DRY_RUN -eq 1 ]]; then
            # Only report truly empty dirs in the dry-run plan.
            if [[ -z "$(ls -A "$month_dir" 2>/dev/null)" ]]; then
              log "PLANNED-RMDIR $month_dir"
            fi
          else
            rmdir "$month_dir" 2>/dev/null && log "RMDIR        $month_dir" || true
          fi
        fi
      done < <(find "$year_dir" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)

      if [[ $DRY_RUN -eq 1 ]]; then
        if [[ -z "$(ls -A "$year_dir" 2>/dev/null)" ]]; then
          log "PLANNED-RMDIR $year_dir"
        fi
      else
        rmdir "$year_dir" 2>/dev/null && log "RMDIR        $year_dir" || true
      fi
    done < <(find "$base" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null)
  done
}

# -------- main -------------------------------------------------------------

if [[ $DRY_RUN -eq 1 ]]; then
  log "### DRY-RUN (no files will be touched). Use --apply to execute."
else
  log "### APPLY mode (files will be moved)."
fi

for pair in "${ROOT_PAIRS[@]}"; do
  root="${pair%%:*}"
  sorted="${pair##*:}"
  log ""
  log "--- root:   $root"
  log "--- sorted: $sorted"
  process_root_pair "$root" "$sorted"
done

log ""
log "--- cleanup phase"
for pair in "${ROOT_PAIRS[@]}"; do
  root="${pair%%:*}"
  sorted="${pair##*:}"
  cleanup_empty_dirs "$root" "$sorted"
done

log ""
log "--- summary"
if [[ $DRY_RUN -eq 1 ]]; then
  log "  planned: $planned"
  log "  planned-dup-trash: $dup_trashed"
  log "  planned-conflicts: $conflicts"
else
  log "  moved: $moved"
  log "  dup-trashed: $dup_trashed"
  log "  conflicts-renamed: $conflicts"
fi
log "  skip-same: $skipped_same"
log "  skip-other: $skipped_other"
log "  warnings: $warnings"
