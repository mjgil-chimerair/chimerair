/*!
 * @file chimera_reader.h
 * @brief File reader component using Chimera ABI
 *
 * C side of the one-binary demo - bounded file reader exporting
 * Chimera-compatible ABI types.
 */

#ifndef CHIMERA_READER_H
#define CHIMERA_READER_H

#include <stdbool.h>
#include <stddef.h>
#include <chimera_abi.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*! @brief Maximum read buffer size */
#define CHIMERA_READER_MAX_SIZE 4096

/*! @brief Reader configuration */
typedef struct chimera_reader_config {
    size_t max_buffer_size;
    bool fail_on_not_found;
} chimera_reader_config_t;

/*! @brief Reader result */
typedef struct chimera_reader_result {
    chimera_status_t status;
    chimera_slice_mut_t buffer;
    size_t bytes_read;
} chimera_reader_result_t;

/*!
 * @brief Create a default reader configuration
 */
CHIMERA_EXPORT chimera_reader_config_t chimera_reader_default_config(void);

/*!
 * @brief Read file contents into buffer
 *
 * @param path File path to read
 * @param config Reader configuration (use default if NULL)
 * @param result Result structure to fill
 * @return CHIMERA_STATUS_OK on success, error code otherwise
 */
CHIMERA_EXPORT chimera_status_t chimera_read_file(
    const char* path,
    chimera_reader_config_t* config,
    chimera_reader_result_t* result
);

/*!
 * @brief Get last error message
 *
 * @return Static error message string
 */
CHIMERA_EXPORT const char* chimera_reader_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* CHIMERA_READER_H */