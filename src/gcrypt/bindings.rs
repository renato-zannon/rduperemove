use libc::{size_t, c_void, c_int, c_uint, c_uchar, c_char};
pub use self::algos::GcryptMdAlgo;

pub static GCRYPT_VERSION: &'static str = "1.6.1";

pub mod algos {
    #[repr(C)]
    pub enum GcryptMdAlgo {
        NONE          = 0,
        MD5           = 1,
        SHA1          = 2,
        RMD160        = 3,
        MD2           = 5,
        TIGER         = 6,   /* TIGER/192 as used by gpg <      = 1.3.2. */
        HAVAL         = 7,   /* HAVAL, 5 pass, 160 bit. */
        SHA256        = 8,
        SHA384        = 9,
        SHA512        = 10,
        SHA224        = 11,
        MD4           = 301,
        CRC32         = 302,
        CRC32_RFC1510 = 303,
        CRC24_RFC2440 = 304,
        WHIRLPOOL     = 305,
        TIGER1        = 306, /* TIGER fixed.  */
        TIGER2        = 307, /* TIGER2 variant.   */
        GOSTR3411_94  = 308, /* GOST R 34.11-94.  */
        STRIBOG256    = 309, /* GOST R 34.11-2012, 256 bit.  */
        STRIBOG512    = 310  /* GOST R 34.11-2012, 512 bit.  */
    }
}

#[repr(C)]
enum GcryCtlCmd {
    /* Note: 1 .. 2 are not anymore used. */
    GCRYCTL_CFB_SYNC                           = 3,
    GCRYCTL_RESET                              = 4,   /* e.g. for MDs */
    GCRYCTL_FINALIZE                           = 5,
    GCRYCTL_GET_KEYLEN                         = 6,
    GCRYCTL_GET_BLKLEN                         = 7,
    GCRYCTL_TEST_ALGO                          = 8,
    GCRYCTL_IS_SECURE                          = 9,
    GCRYCTL_GET_ASNOID                         = 10,
    GCRYCTL_ENABLE_ALGO                        = 11,
    GCRYCTL_DISABLE_ALGO                       = 12,
    GCRYCTL_DUMP_RANDOM_STATS                  = 13,
    GCRYCTL_DUMP_SECMEM_STATS                  = 14,
    GCRYCTL_GET_ALGO_NPKEY                     = 15,
    GCRYCTL_GET_ALGO_NSKEY                     = 16,
    GCRYCTL_GET_ALGO_NSIGN                     = 17,
    GCRYCTL_GET_ALGO_NENCR                     = 18,
    GCRYCTL_SET_VERBOSITY                      = 19,
    GCRYCTL_SET_DEBUG_FLAGS                    = 20,
    GCRYCTL_CLEAR_DEBUG_FLAGS                  = 21,
    GCRYCTL_USE_SECURE_RNDPOOL                 = 22,
    GCRYCTL_DUMP_MEMORY_STATS                  = 23,
    GCRYCTL_INIT_SECMEM                        = 24,
    GCRYCTL_TERM_SECMEM                        = 25,
    GCRYCTL_DISABLE_SECMEM_WARN                = 27,
    GCRYCTL_SUSPEND_SECMEM_WARN                = 28,
    GCRYCTL_RESUME_SECMEM_WARN                 = 29,
    GCRYCTL_DROP_PRIVS                         = 30,
    GCRYCTL_ENABLE_M_GUARD                     = 31,
    GCRYCTL_START_DUMP                         = 32,
    GCRYCTL_STOP_DUMP                          = 33,
    GCRYCTL_GET_ALGO_USAGE                     = 34,
    GCRYCTL_IS_ALGO_ENABLED                    = 35,
    GCRYCTL_DISABLE_INTERNAL_LOCKING           = 36,
    GCRYCTL_DISABLE_SECMEM                     = 37,
    GCRYCTL_INITIALIZATION_FINISHED            = 38,
    GCRYCTL_INITIALIZATION_FINISHED_P          = 39,
    GCRYCTL_ANY_INITIALIZATION_P               = 40,
    GCRYCTL_SET_CBC_CTS                        = 41,
    GCRYCTL_SET_CBC_MAC                        = 42,
    /* Note: 43 is not anymore used. */
    GCRYCTL_ENABLE_QUICK_RANDOM                = 44,
    GCRYCTL_SET_RANDOM_SEED_FILE               = 45,
    GCRYCTL_UPDATE_RANDOM_SEED_FILE            = 46,
    GCRYCTL_SET_THREAD_CBS                     = 47,
    GCRYCTL_FAST_POLL                          = 48,
    GCRYCTL_SET_RANDOM_DAEMON_SOCKET           = 49,
    GCRYCTL_USE_RANDOM_DAEMON                  = 50,
    GCRYCTL_FAKED_RANDOM_P                     = 51,
    GCRYCTL_SET_RNDEGD_SOCKET                  = 52,
    GCRYCTL_PRINT_CONFIG                       = 53,
    GCRYCTL_OPERATIONAL_P                      = 54,
    GCRYCTL_FIPS_MODE_P                        = 55,
    GCRYCTL_FORCE_FIPS_MODE                    = 56,
    GCRYCTL_SELFTEST                           = 57,
    /* Note: 58 .. 62 are used internally.  */
    GCRYCTL_DISABLE_HWF                        = 63,
    GCRYCTL_SET_ENFORCED_FIPS_FLAG             = 64,
    GCRYCTL_SET_PREFERRED_RNG_TYPE             = 65,
    GCRYCTL_GET_CURRENT_RNG_TYPE               = 66,
    GCRYCTL_DISABLE_LOCKED_SECMEM              = 67,
    GCRYCTL_DISABLE_PRIV_DROP                  = 68,
    GCRYCTL_SET_CCM_LENGTHS                    = 69,
    GCRYCTL_CLOSE_RANDOM_DEVICE                = 70,
    GCRYCTL_INACTIVATE_FIPS_FLAG               = 71,
    GCRYCTL_REACTIVATE_FIPS_FLAG               = 72
}

pub type gcrypt_md_handle = *mut u8;

#[link(name = "gcrypt")]
extern "C" {
    pub fn gcry_check_version(req_version: *const c_char) -> *mut c_char;
    pub fn gcry_control(cmd: GcryCtlCmd, ...) -> c_uint;
    pub fn gcry_md_open(h: *mut gcrypt_md_handle, algo: GcryptMdAlgo, flags: c_uint) -> c_uint;
    pub fn gcry_md_write(h: gcrypt_md_handle, buffer: *const c_void, length: size_t) -> c_uint;
    pub fn gcry_md_read(h: gcrypt_md_handle, algo: GcryptMdAlgo) -> *mut c_uchar;
    pub fn gcry_md_close(h: gcrypt_md_handle);
    pub fn gcry_md_get_algo_dlen(algo: GcryptMdAlgo) -> c_int;
    pub fn gcry_md_hash_buffer(algo: GcryptMdAlgo, digest: *mut c_void, buffer: *const c_void, length: size_t);
}

struct GcryptThreadCbs { option: c_uint }

static gcry_threads_pthread: GcryptThreadCbs = GcryptThreadCbs {
    option: (3 | (1 << 8))
};

static gcry_threads_pth: GcryptThreadCbs = GcryptThreadCbs {
    option: (2 | (1 << 8))
};

pub fn init() {
    unsafe {
        gcry_control(GCRYCTL_SET_THREAD_CBS, (&gcry_threads_pthread) as *const GcryptThreadCbs);

        GCRYPT_VERSION.with_c_str(|c_str| {
            gcry_check_version(c_str);
        });
    }
}
