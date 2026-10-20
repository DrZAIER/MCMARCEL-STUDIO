# Locate the McMarcel R implementation, the validation oracle of the rewrite.
#
# The scripts in this folder read the inputs of the original R project. The R
# project is not part of this repository, so it is looked up in this order:
#
#   1. the MCMARCEL_R_ROOT environment variable;
#   2. the parent of the parent (the Studio checked out inside McMarcel);
#   3. the parent (the Studio as a repository root, McMarcel as a sibling);
#   4. a sibling directory called McMarcel or mcmarcel.
resolve_mcmarcel_root <- function(script_path = NULL) {
  if (is.null(script_path)) {
    ofile <- tryCatch(sys.frame(1)$ofile, error = function(e) NULL)
    script_path <- if (!is.null(ofile)) normalizePath(ofile) else normalizePath("parity")
  }
  here <- dirname(script_path)
  candidates <- c(
    Sys.getenv("MCMARCEL_R_ROOT"),
    file.path(here, "..", ".."),
    file.path(here, ".."),
    file.path(here, "..", "McMarcel"),
    file.path(here, "..", "mcmarcel"),
    file.path(here, "..", "..", "McMarcel")
  )
  for (candidate in candidates) {
    if (nzchar(candidate) && file.exists(file.path(candidate, "functions", "RSV_load_all.R"))) {
      return(normalizePath(candidate))
    }
  }
  stop(paste(
    "cannot find the McMarcel R implementation.",
    "Set MCMARCEL_R_ROOT to the directory that contains functions/RSV_load_all.R."
  ))
}
