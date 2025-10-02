#include <stdio.h>
#include <string.h>

int main() {
    printf("Hello!");
    FILE *file_ptr;
    char wav_header_buf[44];
    const char *filename = "/home/riley/music/chocktaw-live.wav";
    char riff_marker[5];

    file_ptr = fopen(filename, "rb");

    size_t header_bytes;
    header_bytes = fread(wav_header_buf, 1, 44, file_ptr);
    memcpy(riff_marker, wav_header_buf, 4);
    riff_marker[4] = '\0';
    printf("Value of the first byte: %s (decimal)\n", riff_marker);
    return 0;
}
