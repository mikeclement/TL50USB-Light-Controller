class Color(object):
    NONE = 0x00
    GREEN = 0x00
    RED = 0x01
    ORANGE = 0x02
    AMBER = 0x03
    YELLOW = 0x04
    LIME_GREEN = 0x05
    SPRING_GREEN = 0x06
    CYAN = 0x07
    SKY_BLUE = 0x08
    BLUE = 0x09
    VIOLET = 0x0a
    MAGENTA = 0x0b
    ROSE = 0x0c
    WHITE = 0x0d


class Intensity(object):
    NONE = 0x00
    HIGH = 0x00
    LOW = 0x01
    MEDIUM = 0x02
    OFF = 0x03


class Animation(object):
    NONE = 0x00
    OFF = 0x00
    STEADY = 0x01
    FLASH = 0x02
    TWO_COLOR_FLASH = 0x03
    HALF_HALF = 0x04
    HALF_HALF_ROTATE = 0x05
    CHASE = 0x06
    INTENSITY_SWEEP = 0x07


class Speed(object):
    NONE = 0x00
    STANDARD = 0x00
    FAST = 0x01
    SLOW = 0x02


class Pattern(object):
    NONE = 0x00
    NORMAL = 0x00
    STROBE = 0x01
    THREE_PULSE = 0x02
    SOS = 0x03
    RANDOM = 0x04


class Rotation(object):
    NONE = 0x00
    COUNTER_CLOCKWISE = 0x00
    CLOCKWISE = 0x01


class Audible(object):
    NONE = 0x00
    OFF = 0x00
    STEADY = 0x01
    PULSED = 0x02
    SOS = 0x03


def compute_checksum(commands):
    """
    compute_checksum calculates the correct checksum based on the Banner Engineering serial protocol documentation
    
    returns: a list with two hex bytes
    """
    
    command_total = sum(commands)
    ones_comp = command_total ^ 0xFFFF
    
    byte1 = (ones_comp & 0xff00) >> 8
    byte2 = (ones_comp & 0x00ff)
    return [byte2, byte1]


def build_command_bytearray(color1, intensity1, animation, speed, pattern, color2, intensity2, rotation, audible):
    """
    build_command_bytearray creates a 38 byte command to send to the TL50 light via a serial protocol. Documentation
    can be found here: https://info.bannerengineering.com/cs/groups/public/documents/literature/218025.pdf
    
    returns: a bytearray of length 38 bytes, including a checksum.

    Note:
    for the Chase animation: color #2 is the background, color #1 is the moving color 
    """

    # The first five bytes are hard-coded
    command_string = [0xF4, 0x41, 0xC1, 0x1F, 0x00] + 31 * [0x00]

    # The sixth byte is a bitfield
    # 1-4: color code 1
    # 5-7: intensity code 1
    # 8: reserved as 0
    command_string[5] = (color1 & 0xf) | ((intensity1 & 0x7) << 4)

    # The seventh byte is a bitfield
    # 1-3: animation code
    # 4-5: speed code
    # 6-8: pattern code
    command_string[6] = (animation & 0x7) | ((speed & 0x3) << 3) | ((pattern & 0x7) << 5)

    # The eighth byte is a bitfield
    # 1-4: color code 2
    # 5-7: intensity code 2
    # 8: rotation direction code
    command_string[7] = (color2 & 0xf) | ((intensity2 & 0x7) << 4) | ((rotation & 0x1) << 7)

    # The ninth through 35th bytes are 0x00

    # The 36th byte is an audible code
    command_string[35] = audible

    # The 37th and 38th bytes are the checksum
    checksum = compute_checksum(command_string)

    return bytearray(command_string + checksum)


def off():
    return build_command_bytearray(Color.NONE, Intensity.OFF, Animation.NONE, Speed.NONE, Pattern.NONE, Color.NONE,
                                   Intensity.NONE, Rotation.NONE, Audible.NONE)

def steady(color, intensity):
    return build_command_bytearray(color, intensity, Animation.STEADY, Speed.NONE, Pattern.NONE, Color.NONE,
                                   Intensity.NONE, Rotation.NONE, Audible.NONE)

def flash(color, intensity, speed, pattern):
    return build_command_bytearray(color, intensity, Animation.FLASH, speed, pattern, Color.NONE,
                                   Intensity.NONE, Rotation.NONE, Audible.NONE)

def two_color_flash(color1, intensity1, color2, intensity2, speed, pattern):
    return build_command_bytearray(color1, intensity1, Animation.TWO_COLOR_FLASH, speed, pattern, color2,
                                   intensity2, Rotation.NONE, Audible.NONE)

def half_half(color1, intensity1, color2, intensity2):
    return build_command_bytearray(color1, intensity1, Animation.HALF_HALF, Speed.NONE, Pattern.NONE, color2,
                                   intensity2, Rotation.NONE, Audible.NONE)

def half_half_rotate(color1, intensity1, color2, intensity2, speed, rotation):
    return build_command_bytearray(color1, intensity1, Animation.HALF_HALF_ROTATE, speed, Pattern.NONE, color2,
                                   intensity2, rotation, Audible.NONE)

def chase(color1, intensity1, color2, intensity2, speed, rotation):
    return build_command_bytearray(color1, intensity1, Animation.CHASE, speed, Pattern.NONE, color2,
                                   intensity2, rotation, Audible.NONE)


if __name__ == '__main__':
    import serial
    import time

    ser = serial.Serial()
    ser.baudrate = 19200
    ser.port = '/dev/ttyUSB0'

    with ser as s:
        def change(cmd, secs):
            s.write(cmd)
            s.flush()
            time.sleep(secs)

        change(steady(Color.GREEN, Intensity.MEDIUM), 2)
        change(flash(Color.AMBER, Intensity.HIGH, Speed.STANDARD, Pattern.STROBE), 2)
        change(two_color_flash(Color.YELLOW, Intensity.HIGH, Color.BLUE, Intensity.HIGH, Speed.FAST, Pattern.NORMAL), 2)
        change(half_half(Color.SPRING_GREEN, Intensity.HIGH, Color.MAGENTA, Intensity.HIGH), 2)
        change(half_half_rotate(Color.SPRING_GREEN, Intensity.HIGH, Color.MAGENTA, Intensity.HIGH, Speed.FAST, Rotation.CLOCKWISE), 2)
        change(chase(Color.SPRING_GREEN, Intensity.HIGH, Color.MAGENTA, Intensity.HIGH, Speed.SLOW, Rotation.COUNTER_CLOCKWISE), 2)
        change(off(), 0)
