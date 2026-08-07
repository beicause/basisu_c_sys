static int __errno_storage = 0;

int *__errno(void)
{
    return &__errno_storage;
}

int *__errno_location(void)
{
    return &__errno_storage;
}

/* musl's internal errno macro (src/include/errno.h) maps errno to the
 * triple-underscore spelling; musl-internal objects reference this one. */
int *___errno_location(void)
{
    return &__errno_storage;
}
