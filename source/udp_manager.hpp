// Most of the UDP code comes from hid-mitm: https://github.com/jakibaki/hid-mitm

extern "C" {
    #include <switch.h>
    #define INPUT_MSG_MAGIC 0x3276

    //Controller Types:
    //0 - none (disconnect controller from switch)
    //1 - Pro Controller
    //2 - Joy-Con (L sideways)
    //3 - Joy-Con (R sideways)
    // TO BE ADDED:
    //4 - Joy-Cons (L and R)
    //5 - Joy-Con (L)
    //6 - Joy-Con (R)

    struct __attribute__((__packed__)) input_message
    {
    public:
        u16 magic;
        u16 con_count;
        u16 con_type;
        u64 keys;
        s32 joy_l_x;
        s32 joy_l_y;
        s32 joy_r_x;
        s32 joy_r_y;
        u16 con_type2;
        u64 keys2;
        s32 joy_l_x2;
        s32 joy_l_y2;
        s32 joy_r_x2;
        s32 joy_r_y2;
        u16 con_type3;
        u64 keys3;
        s32 joy_l_x3;
        s32 joy_l_y3;
        s32 joy_r_x3;
        s32 joy_r_y3;
        u16 con_type4;
        u64 keys4;
        s32 joy_l_x4;
        s32 joy_l_y4;
        s32 joy_r_x4;
        s32 joy_r_y4;
        u16 con_type5;
        u64 keys5;
        s32 joy_l_x5;
        s32 joy_l_y5;
        s32 joy_r_x5;
        s32 joy_r_y5;
        u16 con_type6;
        u64 keys6;
        s32 joy_l_x6;
        s32 joy_l_y6;
        s32 joy_r_x6;
        s32 joy_r_y6;
        u16 con_type7;
        u64 keys7;
        s32 joy_l_x7;
        s32 joy_l_y7;
        s32 joy_r_x7;
        s32 joy_r_y7;
        u16 con_type8;
        u64 keys8;
        s32 joy_l_x8;
        s32 joy_l_y8;
        s32 joy_r_x8;
        s32 joy_r_y8;
    };

    int poll_udp_input(input_message* buf);
    void networkThread(void* _);
}
