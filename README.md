# Nucleo_F4xx_PlantWelfare
control pumps and read moisture sensors with a f401 or f427 nucleo board

### current opperating modes

 * The pumps are activated on a time scedule 
    * currently the activation time is fixed 
 * the serial line over the st-link is meant as command interface and for configuing purposes 
 * the button activates the pumps one after another  

### planned features 
 
 * have a configuration for each plant (name, water amount, water scedule?, low/high Level)
 * actually use the moiture sensors
 * have some sort of website (Wlan module) to monitor the values when on vacation 
   And to sync the rtc time with the internet

#### Flash using Probe.rs

```cargo flash --chip stm32f401re```

Or with cargo embed

```cargo embed --release ```

If probe fails to flash your board you probably need to update the firmware on the onboard programmer.
The updater can be found at: https://www.st.com/en/development-tools/stsw-link007.html

### View with Rtt output 

RTT uses the ITM Interface. Rust is able to print the swv output via: 

```cargo embed --release```

the flash command only transmit the frimware image. 

### The Water sensor 

The chip on the sensor modul generate an analog voltage according to the moisture level.

A dry plant had about 2300mV
A freash watered one was at 1300mV

### Board properties

 * User led on PA5
 * User button on PC13
 * Serial port through ST-LINK on USART2, Tx: PA2 and Rx: PA3.

This repository is based on https://github.com/jkristell/nucleo-f401re