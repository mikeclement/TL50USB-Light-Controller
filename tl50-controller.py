
import serial
import binascii
import time
from optparse import OptionParser

cmd_checksum = [0x00, 0x00]

def compute_checksum(commands):
    """
    compute_checksum calcuates the correct checksum based on the Banner Engineering serial protocol documentation
    
    returns: a list with two hex bytes
    """
    
    command_total = sum(commands)
    ones_comp = command_total ^ 0xFFFF
    
    byte1 = (ones_comp & 0xff00) >>8
    byte2 = (ones_comp & 0x00ff)
    #print(hex(byte1) + ':'+hex(byte2))
    return [byte2, byte1] 


def set_segment(**kwargs):
    """
    set_segment creates a 30 byte hex command to send to the TL50 light via a USB serial using the following minimum inputs:
    
    - intensity = see color_intensity_index
    - color_num_one = see colors_index
    - color_num_two = see colors_index
    - animation = see animation_index
    
    returns (default): a binary formatted hex string of 30 bytes, including a checksum.
    returns (debug mode): a binary formatted hex string of 30 bytes, including a checksum, but using a static debug_string list of hex bytes instead of a generated one.
    
    Note:
    for the Chase animation: color #2 is the background, color #1 is the moving color 
    """
    command_string = [0xF4, 0x41, 0xC1, 0x1F, 0x00, 0x0d, 0x00, 0xd, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    
    debug_string =   [0xF4, 0x41, 0xC1, 0x1F, 0x00, 0x00, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    
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
    
    color_inten = color_intensity_index.get(kwargs['intensity'], 0x00)
        
    color_one = colors_index.get(kwargs['color_num_one'], 0xd)
    color_two = colors_index.get(kwargs['color_num_two'], 0xd)
    
    command_string[5] = color_one #color one 
    command_string[7] = color_two #color two
    
    animation_type = animation_index.get(kwargs['animation'], 0x00)
    #set the 6th byte to the correct hex string 
    command_string[6] = animation_type
    
    #generate the array
    if options.debug == True:
        light_command = bytearray( debug_string + compute_checksum(debug_string) )
    else:
        light_command = bytearray( command_string + compute_checksum(command_string) )
    
    if options.debug == True:
        print("Args are the following:")
        print(kwargs)
        #print("Byte Command")
        #print(light_command)
        print("Light Number:")
        print(color_num)
        
    return light_command  

#CLI Parsing Init
parser = OptionParser()
parser.add_option("--debug", action="store_true", dest="debug")

(options, args) = parser.parse_args() #create a varible to store the state of the debug command
   
#Create a new serial object and set the baudrate and port   
ser = serial.Serial()
ser.baudrate = 19200
#ser.write_timeout=2
#ser.inter_byte_timeout=2

ser1 = serial.Serial()
ser1.baudrate = 19200

#single hight frosted
ser.port = '/dev/tty.usbserial-FT791OX9'

#double hight frosted
#ser.port = '/dev/tty.usbserial-FT7BVSDW'

#clear light
#ser1.port= '/dev/tty.usbserial-FT4VLXSZ'
    
ser.open()
#ser1.open()

if ser.is_open:    
    #ser.write(set_segment(color_num="1", intensity="high", color="green", animation="intens_sweep"))
    #ser1.write(set_segment(color_num="1", intensity="high", color="green", animation="flash"))
    #time.sleep(0.09)
    #time.sleep(5)
    
    ser.write(set_segment(intensity="high", color_num_one="rose", color_num_two="blue", animation="two_color_flash"))
    time.sleep(0.09)
    #time.sleep(5)
    
    ser.close()