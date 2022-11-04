import serial
import binascii
import time

command_string = [0xF4, 0x41, 0xC1, 0x1F, 0x00, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
cmd_checksum = [0x00, 0x00]

def set_animation(animation, cmd_string):
    animation_index = {
      "off": 0x00,
      "steady": 0x01,
      "flash": 0x02,
      "two_color_flash": 0x03,
      "half_half":0x04,
      "half_half_rot": 0x05,
      "chase": 0x06,
      "intens_sweep": 0x07
    }
    
    animation_type = animation_index.get(animation, 0x00)
    #set the 5th byte to the correct  
    cmd_string[6] = animation_type

def set_color(color, color_num, intensity, cmd_string):
    colors_index = {
      "green": 0x00,
      "red": 0x01,
      "orange": 0x02,
      "amber": 0x03,
      "yellow":0x04,
      "lime_green": 0x05,
      "spring_green": 0x06,
      "cyan": 0x07,
      "sky_blue": 0x08,
      "blue": 0x09,
      "violet":0xa,
      "magenta":0xb,
      "rose": 0xc,
      "white": 0xd
    }
    
    color_intensity_index = {
      "high": 0x00,
      "low": 0x01,
      "medium": 0x02,
      "off": 0x03,
    }
    
    color_inten = color_intensity_index.get(intensity, 0x00)
    color_hex = colors_index.get(color, 0x00)
    #set the 5th byte to the correct  
    if color_num == 1:
        #cmd_string[5] = color_hex
        cmd_string[5] = color_hex
    if color_num == 2:  
        cmd_string[7] = color_hex
    
    

def compute_checksum(commands):
    command_total = sum(commands)
    ones_comp = command_total ^ 0xFFFF
    
    byte1 = (ones_comp & 0xff00) >>8
    byte2 = (ones_comp & 0x00ff)
    #print(hex(byte1) + ':'+hex(byte2))
    return [byte2, byte1]

def gen_light_cmd(cmd_string):
    light_command = bytearray( cmd_string + compute_checksum(cmd_string) )
    return light_command   

   
ser = serial.Serial()
ser.baudrate = 19200

#single hight frosted
#ser.port = '/dev/tty.usbserial-FT791OX9'

#double hight frosted
#ser.port = '/dev/tty.usbserial-FT7BVSDW'

#clear light
ser.port= '/dev/tty.usbserial-FT4VLXSZ'
    
ser.open()

set_color('blue', 1, 'low', command_string)
#set_color('orange', 1, command_string)
#set_color('green', 2, command_string)
#set_animation('half_half_rot', command_string)
set_animation('steady', command_string)

if ser.is_open:
    #ser.reset_output_buffer()
    
    #Solid Blue
    set_animation('steady', command_string)
    set_color('blue', 1, 'high', command_string)    
    light_command = bytearray(command_string+compute_checksum(command_string))
    ser.write( gen_light_cmd(command_string) )
    time.sleep(0.09)
    
    time.sleep(5)
    
    set_animation('flash', command_string)
    set_color('blue', 1, 'high', command_string)    
    light_command = bytearray(command_string+compute_checksum(command_string))
    ser.write( gen_light_cmd(command_string) )
    time.sleep(0.09)
    
    time.sleep(5)
    
    #Two Tone Green Rotation
    set_animation('half_half_rot', command_string)
    set_color('green', 1, 'high', command_string)
    set_color('lime_green', 2, 'high', command_string)    
    ser.write(gen_light_cmd(command_string))
    time.sleep(0.09)
    
    time.sleep(5)
    
    #Green Pulsing
    set_animation('intens_sweep', command_string)
    set_color('green', 1, 'high', command_string)
    ser.write(gen_light_cmd(command_string))
    time.sleep(0.09)
    
    time.sleep(5)
    
    #Green and Red Two Color Flash
    set_animation('two_color_flash', command_string)
    set_color('green', 1, 'high', command_string)
    set_color('red', 2, 'high', command_string)    
    ser.write(gen_light_cmd(command_string))
    time.sleep(0.09)
    
    time.sleep(5)
    
    #Red and Green Chase
    set_animation('chase', command_string)
    set_color('orange', 1, 'high', command_string)
    set_color('green', 2, 'high', command_string)    
    ser.write(gen_light_cmd(command_string))
    time.sleep(0.09)
    
    time.sleep(5)
    
    #Turn off light
    set_color('green', 1, 'high', command_string)
    set_animation('off', command_string)
    ser.write(gen_light_cmd(command_string))
    time.sleep(0.09)
    
    #ser.flush()

    ser.close()