#include "con_manager.hpp"
#include "udp_manager.hpp"
#include <mutex>
#include <array>

// Some of the code comes from hid-mitm

int FakeController::initialize(u16 conDeviceType)
{
    if (isInitialized) return 0;
    Result myResult;
    //printToFile("Controller initializing...");

    // Set the controller type to Pro-Controller, and set the npadInterfaceType.
    

    switch(conDeviceType)
    {
        case 1:
            controllerDevice.deviceType = HidDeviceType_FullKey3; // Pro Controller
            break;
        
        case 2:
            controllerDevice.deviceType = HidDeviceType_JoyLeft2; // Joy-Con Left
            break;

        case 3:
            controllerDevice.deviceType = HidDeviceType_JoyRight1; // Joy-Con Right
            break;

    }

    // Set the controller colors. The grip colors are for Pro-Controller on [9.0.0+].
    controllerDevice.singleColorBody = RGBA8_MAXALPHA(255,153,204);
    controllerDevice.singleColorButtons = RGBA8_MAXALPHA(0,0,0);
    if (conDeviceType == 1)
    {
        controllerDevice.colorLeftGrip = RGBA8_MAXALPHA(255,0,127);
        controllerDevice.colorRightGrip = RGBA8_MAXALPHA(255,0,127);
    }
    
    controllerDevice.npadInterfaceType = HidNpadInterfaceType_Bluetooth;

    // Setup example controller state.
    controllerState.battery_level = 4; // Set battery charge to full.

    if (conDeviceType == 1 || conDeviceType == 2)
    {
        controllerState.analog_stick_l.x = 0x0;
        controllerState.analog_stick_l.y = -0x0;
    }

    if (conDeviceType == 1 || conDeviceType == 3)
    {
        controllerState.analog_stick_r.x = 0x0;
        controllerState.analog_stick_r.y = -0x0;
    }
    
    myResult = hiddbgAttachHdlsVirtualDevice(&controllerHandle, &controllerDevice);
    if (R_FAILED(myResult)) {
        printToFile("Failed connecting controller... fuck");
        return -1;
    }

    printToFile("Controller initialized!");
    isInitialized = true;
    return 0;
}

int FakeController::deInitialize()
{
    if (!isInitialized) return 0;
    Result myResult;

    controllerState = {0};
    hiddbgSetHdlsState(controllerHandle, &controllerState);
    
    myResult = hiddbgDetachHdlsVirtualDevice(controllerHandle);
    if (R_FAILED(myResult)) {
        printToFile("Fatal Error while detaching controller.");
    }
    controllerHandle = {0};
    controllerDevice = {0};

    isInitialized = false;

    return 0;
}

std::array<FakeController, 8> fakeControllerList;
u64 buttonPresses;

void apply_fake_con_state(struct input_message message)
{
    // Check if the magic is correct
    if(message.magic != INPUT_MSG_MAGIC)
        return;

    u16 conType;
    u64 keys;
    s32 joylx;
    s32 joyly;
    s32 joyrx;
    s32 joyry;

    for(s32 i = 0; i < message.con_count; i++)
    {
        switch(i)
        {
            case 0:
                conType = message.con_type;
                keys = message.keys;
                joylx = message.joy_l_x;
                joyly = message.joy_l_y;
                joyrx = message.joy_r_x;
                joyry = message.joy_r_y;
                break;
            case 1:
                conType = message.con_type2;
                keys = message.keys2;
                joylx = message.joy_l_x2;
                joyly = message.joy_l_y2;
                joyrx = message.joy_r_x2;
                joyry = message.joy_r_y2;
                break;
            case 2:
                conType = message.con_type3;
                keys = message.keys3;
                joylx = message.joy_l_x3;
                joyly = message.joy_l_y3;
                joyrx = message.joy_r_x3;
                joyry = message.joy_r_y3;
                break;
            case 3:
                conType = message.con_type4;
                keys = message.keys4;
                joylx = message.joy_l_x4;
                joyly = message.joy_l_y4;
                joyrx = message.joy_r_x4;
                joyry = message.joy_r_y4;
                break;
            case 4:
                conType = message.con_type5;
                keys = message.keys5;
                joylx = message.joy_l_x5;
                joyly = message.joy_l_y5;
                joyrx = message.joy_r_x5;
                joyry = message.joy_r_y5;
                break;
            case 5:
                conType = message.con_type6;
                keys = message.keys6;
                joylx = message.joy_l_x6;
                joyly = message.joy_l_y6;
                joyrx = message.joy_r_x6;
                joyry = message.joy_r_y6;
                break;
            case 6:
                conType = message.con_type7;
                keys = message.keys7;
                joylx = message.joy_l_x7;
                joyly = message.joy_l_y7;
                joyrx = message.joy_r_x7;
                joyry = message.joy_r_y7;
                break;
            case 7:
                conType = message.con_type8;
                keys = message.keys8;
                joylx = message.joy_l_x8;
                joyly = message.joy_l_y8;
                joyrx = message.joy_r_x8;
                joyry = message.joy_r_y8;
                break;
        }

        // If there is no controller connected, we have to initialize one
        if (!fakeControllerList[i].isInitialized && (conType > 0 && conType < 4))
        {
            fakeControllerList[i].initialize(conType);
        } 
        // If there is a controller connected, but we changed the controller type to a non-existant one, we'll disconnect it
        else if (fakeControllerList[i].isInitialized && (conType < 1 || conType > 3))
        {
            fakeControllerList[i].deInitialize();
            /*FakeController tempCon;
            fakeControllerList[i] = tempCon;*/
        }

        if (fakeControllerList[i].isInitialized)
        {
            fakeControllerList[i].controllerState.buttons = keys;
            fakeControllerList[i].controllerState.analog_stick_l.x = joylx;
            fakeControllerList[i].controllerState.analog_stick_l.y = joyly;
            fakeControllerList[i].controllerState.analog_stick_r.x = joyrx;
            fakeControllerList[i].controllerState.analog_stick_r.y = joyry;
            Result myResult;
            // This function is causing all the issues in 12.0
            myResult = hiddbgSetHdlsState(fakeControllerList[i].controllerHandle, &fakeControllerList[i].controllerState);
            if (R_FAILED(myResult)) {
                printToFile("Fatal Error while updating Controller State.");
            }   
        }
    }
    
    return;
}

static Mutex pkgMutex;
static struct input_message fakeConsState;

void networkThread(void* _)
{
    struct input_message temporal_pkg;
    printToFile("Starting Network Loop Thread!");
    while (true)
    {
        int poll_res = poll_udp_input(&temporal_pkg);
        mutexLock(&pkgMutex);

        if (poll_res == 0)
        {
            fakeConsState = temporal_pkg;
            apply_fake_con_state(fakeConsState);
        }
        else
        {
            fakeConsState.magic = 0;
            svcSleepThread(1e+7l);
        }
        mutexUnlock(&pkgMutex);

        svcSleepThread(-1);
    }
}
