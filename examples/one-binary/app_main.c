#include "c-reader/chimera_reader.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

size_t chimera_rust_count_config_entries(const uint8_t* data, size_t len);
uint32_t chimera_zig_crc32(const uint8_t* data, size_t len);

int main(int argc, char** argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: %s <config-file>\n", argv[0]);
        return 2;
    }

    chimera_reader_config_t config = chimera_reader_default_config();
    chimera_reader_result_t result;
    chimera_status_t status = chimera_read_file(argv[1], &config, &result);
    if (status != CHIMERA_STATUS_OK) {
        fprintf(stderr, "read error: %s\n", chimera_reader_last_error());
        return 1;
    }

    size_t entry_count = chimera_rust_count_config_entries(
        (const uint8_t*)result.buffer.data,
        result.buffer.len
    );
    uint32_t checksum = chimera_zig_crc32(
        (const uint8_t*)result.buffer.data,
        result.buffer.len
    );

    printf("entries=%zu\n", entry_count);
    printf("checksum=%u\n", checksum);

    free(result.buffer.data);
    return 0;
}
