#include "chimera_reader.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char g_last_error[128] = "";

static void set_last_error(const char* message) {
    snprintf(g_last_error, sizeof(g_last_error), "%s", message);
}

chimera_reader_config_t chimera_reader_default_config(void) {
    chimera_reader_config_t config;
    config.max_buffer_size = CHIMERA_READER_MAX_SIZE;
    config.fail_on_not_found = true;
    return config;
}

chimera_status_t chimera_read_file(
    const char* path,
    chimera_reader_config_t* config,
    chimera_reader_result_t* result
) {
    chimera_reader_config_t effective_config;
    FILE* file;
    long file_size;
    uint8_t* buffer;

    if (path == NULL || result == NULL) {
        set_last_error("Invalid argument");
        return CHIMERA_STATUS_INVALID_ARG;
    }

    effective_config = config != NULL ? *config : chimera_reader_default_config();
    result->status = CHIMERA_STATUS_OK;
    result->buffer.data = NULL;
    result->buffer.len = 0;
    result->bytes_read = 0;

    file = fopen(path, "rb");
    if (file == NULL) {
        if (effective_config.fail_on_not_found) {
            set_last_error("Failed to open file");
            result->status = CHIMERA_STATUS_NOT_FOUND;
            return CHIMERA_STATUS_NOT_FOUND;
        }
        return CHIMERA_STATUS_OK;
    }

    if (fseek(file, 0, SEEK_END) != 0) {
        fclose(file);
        set_last_error("Failed to get file size");
        return CHIMERA_STATUS_ERROR;
    }

    file_size = ftell(file);
    if (file_size < 0) {
        fclose(file);
        set_last_error("Failed to get file size");
        return CHIMERA_STATUS_ERROR;
    }

    if ((size_t)file_size > effective_config.max_buffer_size) {
        fclose(file);
        set_last_error("Buffer too small");
        return CHIMERA_STATUS_BUFFER_TOO_SMALL;
    }

    if (fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        set_last_error("Failed to get file size");
        return CHIMERA_STATUS_ERROR;
    }

    if (file_size == 0) {
        fclose(file);
        return CHIMERA_STATUS_OK;
    }

    buffer = (uint8_t*)malloc((size_t)file_size);
    if (buffer == NULL) {
        fclose(file);
        set_last_error("Out of memory");
        return CHIMERA_STATUS_OUT_OF_MEMORY;
    }

    if (fread(buffer, 1, (size_t)file_size, file) != (size_t)file_size) {
        free(buffer);
        fclose(file);
        set_last_error("Read error");
        return CHIMERA_STATUS_ERROR;
    }

    fclose(file);
    result->buffer.data = buffer;
    result->buffer.len = (size_t)file_size;
    result->bytes_read = (size_t)file_size;
    return CHIMERA_STATUS_OK;
}

const char* chimera_reader_last_error(void) {
    return g_last_error;
}
