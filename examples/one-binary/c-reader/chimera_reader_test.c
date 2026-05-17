#include "chimera_reader.h"

#include <stdio.h>

static int tests_passed = 0;
static int tests_failed = 0;

static void pass(const char* message) {
    printf("  PASS: %s\n", message);
    tests_passed++;
}

static void fail(const char* message) {
    printf("  FAIL: %s\n", message);
    tests_failed++;
}

static void test_default_config(void) {
    chimera_reader_config_t config = chimera_reader_default_config();

    printf("Test: default_config\n");
    if (config.max_buffer_size == CHIMERA_READER_MAX_SIZE) {
        pass("max_buffer_size matches constant");
    } else {
        fail("max_buffer_size matches constant");
    }

    if (config.fail_on_not_found) {
        pass("fail_on_not_found is true by default");
    } else {
        fail("fail_on_not_found is true by default");
    }
}

static void test_read_nonexistent_file(void) {
    chimera_reader_config_t config = chimera_reader_default_config();
    chimera_reader_result_t result;
    chimera_status_t status;

    printf("Test: read_nonexistent_file\n");
    config.fail_on_not_found = false;
    status = chimera_read_file("/nonexistent/file/path.txt", &config, &result);

    if (status == CHIMERA_STATUS_OK) {
        pass("returns OK when fail_on_not_found is false");
    } else {
        fail("returns OK when fail_on_not_found is false");
    }

    if (result.buffer.data == NULL) {
        pass("buffer is NULL for nonexistent file");
    } else {
        fail("buffer is NULL for nonexistent file");
    }

    if (result.bytes_read == 0) {
        pass("bytes_read is 0");
    } else {
        fail("bytes_read is 0");
    }
}

static void test_read_missing_file_with_fail(void) {
    chimera_reader_config_t config = chimera_reader_default_config();
    chimera_reader_result_t result;
    chimera_status_t status;

    printf("Test: read_missing_file_with_fail\n");
    status = chimera_read_file("/nonexistent/file/path.txt", &config, &result);

    if (status == CHIMERA_STATUS_NOT_FOUND) {
        pass("returns NOT_FOUND status");
    } else {
        fail("returns NOT_FOUND status");
    }

    if (result.buffer.data == NULL) {
        pass("buffer is NULL");
    } else {
        fail("buffer is NULL");
    }
}

int main(void) {
    printf("Running chimera_reader tests...\n");
    test_default_config();
    test_read_nonexistent_file();
    test_read_missing_file_with_fail();
    printf("\nResults: %d passed, %d failed\n", tests_passed, tests_failed);
    return tests_failed == 0 ? 0 : 1;
}
