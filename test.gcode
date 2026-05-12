G90 ; Absolute positioning
G1 F3000 ; Set feedrate to 3000 mm/min (50 mm/s)
G1 X0 Y0 Z0 ; Go to Front-Left corner at bed level
G1 X100 Y0 Z10 ; Move right and up
G1 X100 Y100 Z10 ; Move back
G1 X0 Y100 Z0 ; Move left and down
G1 X0 Y0 Z0 ; Return to start
G1 F1000 ; Slower move
G1 X200 Y200 Z50 ; Move to far corner
