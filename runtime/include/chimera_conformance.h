/*!
 * @file chimera_conformance.h
 * @brief Runtime conformance test suite
 *
 * Tests that C, Rust, and Zig runtime artifacts agree on struct layout,
 * constants, and calling conventions.
 */

#ifndef CHIMERA_CONFORMANCE_H
#define CHIMERA_CONFORMANCE_H

#include <chimera_abi.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*============================================================================
 * Status Code Conformance
 *============================================================================*/

/*!
 * @brief Check status code values match expected
 * @return true if all status codes match C enum values
 */
CHIMERA_EXPORT bool chimera_conformance_status_codes(void);

/*!
 * @brief Check status code display strings
 * @return true if all status codes have valid display strings
 */
CHIMERA_EXPORT bool chimera_conformance_status_display(void);

/*============================================================================
 * Struct Layout Conformance
 *============================================================================*/

/*!
 * @brief Check Slice struct layout
 * @return true if layout matches expected (data:ptr, len:usize)
 */
CHIMERA_EXPORT bool chimera_conformance_slice_layout(void);

/*!
 * @brief Check SliceMut struct layout
 * @return true if layout matches expected (data:ptr, len:usize)
 */
CHIMERA_EXPORT bool chimera_conformance_slice_mut_layout(void);

/*!
 * @brief Check Result struct layout
 * @return true if layout matches expected (is_ok:bool + padding)
 */
CHIMERA_EXPORT bool chimera_conformance_result_layout(void);

/*!
 * @brief Check Error struct layout
 * @return true if layout matches expected field offsets
 */
CHIMERA_EXPORT bool chimera_conformance_error_layout(void);

/*============================================================================
 * Constant Conformance
 *============================================================================*/

/*!
 * @brief Check ABI version constants
 * @return true if version constants match expected values
 */
CHIMERA_EXPORT bool chimera_conformance_version(void);

/*!
 * @brief Check Ownership enum values
 * @return true if ownership values match
 */
CHIMERA_EXPORT bool chimera_conformance_ownership(void);

/*!
 * @brief Check Lifetime enum values
 * @return true if lifetime values match
 */
CHIMERA_EXPORT bool chimera_conformance_lifetime(void);

/*!
 * @brief Check CConv enum values
 * @return true if calling convention values match
 */
CHIMERA_EXPORT bool chimera_conformance_cconv(void);

/*============================================================================
 * Size Conformance
 *============================================================================*/

/*!
 * @brief Check that sizeof(Slice) is consistent
 * @return true if size is as expected
 */
CHIMERA_EXPORT bool chimera_conformance_slice_size(void);

/*!
 * @brief Check that sizeof(SliceMut) is consistent
 * @return true if size is as expected
 */
CHIMERA_EXPORT bool chimera_conformance_slice_mut_size(void);

/*!
 * @brief Check that sizeof(Result) is consistent
 * @return true if size is as expected
 */
CHIMERA_EXPORT bool chimera_conformance_result_size(void);

/*============================================================================
 * Alignment Conformance
 *============================================================================*/

/*!
 * @brief Check Slice alignment requirements
 * @return true if alignment is correct
 */
CHIMERA_EXPORT bool chimera_conformance_slice_alignment(void);

/*!
 * @brief Check SliceMut alignment requirements
 * @return true if alignment is correct
 */
CHIMERA_EXPORT bool chimera_conformance_slice_mut_alignment(void);

/*============================================================================
 * Full Conformance Suite
 *============================================================================*/

/*!
 * @brief Run full conformance suite
 * @return true if all conformance checks pass
 */
CHIMERA_EXPORT bool chimera_conformance_run_all(void);

/*!
 * @brief Get conformance test name by index
 * @param index Test index
 * @return Static test name string or NULL if out of range
 */
CHIMERA_EXPORT const char* chimera_conformance_get_test_name(size_t index);

/*!
 * @brief Get total number of conformance tests
 * @return Number of tests in suite
 */
CHIMERA_EXPORT size_t chimera_conformance_test_count(void);

#ifdef __cplusplus
}
#endif

#endif /* CHIMERA_CONFORMANCE_H */