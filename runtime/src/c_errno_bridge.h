//! C Errno Bridge Runtime Helpers (Task 119)
//!
//! This module provides helper functions that map errno and C error domains
//! into the canonical ch_error_t structure defined in chimera_abi.h.

#include <errno.h>
#include <stdint.h>

// Include the canonical ABI header
#include "chimera_abi.h"

/*!
 * @brief Map POSIX errno to a chimera domain code.
 *
 * This helper converts standard POSIX errno values to their
 * corresponding chimera error domain and code representation.
 *
 * @param posix_errno The POSIX errno value (e.g., EINVAL, ENOMEM)
 * @param out_error Output parameter for the mapped ch_error_t
 *
 * Usage:
 *   ch_error_t err;
 *   chimera_errno_to_error(EINVAL, &err);
 */
static inline void chimera_errno_to_error(int posix_errno, ch_error_t* out_error) {
    if (out_error == NULL) return;

    // Map POSIX errno to chimera domain
    // The chimera domain uses a specific numeric space for C/POSIX errors
    out_error->domain = 1;  // CHIMERA_DOMAIN_POSIX
    out_error->code = (uint32_t)posix_errno;
    out_error->flags = (posix_errno == 0) ? 0 : 1;  // 1 = error flag
}

/*!
 * @brief Check if a POSIX errno value indicates success.
 *
 * In POSIX, 0 is success and everything else is an error.
 * This helper encapsulates that check.
 *
 * @param posix_errno The POSIX errno value
 * @return true if errno indicates success
 */
static inline bool chimera_errno_is_ok(int posix_errno) {
    return posix_errno == 0;
}

/*!
 * @brief Convert ch_status to a printable error string.
 *
 * Returns a constant string describing the status code.
 *
 * @param status The ch_status value
 * @return Constant string description
 */
static inline const char* chimera_status_to_string(ch_status status) {
    switch (status) {
        case CHIMERA_STATUS_OK: return "OK";
        case CHIMERA_STATUS_ERROR: return "ERROR";
        case CHIMERA_STATUS_INVALID_ARG: return "INVALID_ARG";
        case CHIMERA_STATUS_INVALID_STATE: return "INVALID_STATE";
        case CHIMERA_STATUS_NOT_FOUND: return "NOT_FOUND";
        default: return "UNKNOWN";
    }
}

/*!
 * @brief Create a ch_error_t from a POSIX errno value.
 *
 * This is a convenience wrapper that combines the domain mapping
 * and error structure initialization.
 *
 * @param posix_errno The POSIX errno value
 * @return A fully initialized ch_error_t structure
 */
static inline ch_error_t chimera_error_from_errno(int posix_errno) {
    ch_error_t err = {0};
    chimera_errno_to_error(posix_errno, &err);
    return err;
}