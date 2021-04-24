rust stm32f3 hc-sr04(ultrasonic ranging module)

implemented on TIM4 where CH1 is used for capture(pwm input) and the CH3 is used to generate control signal(pwm output).

CH3 generate impulse 10us width with 50ms period. 
CH1 is configured for both signal edges(rise, fall) and by those values input signal impulse width is measured.

I use "saleae logic" for signals capture.

pwm output
![alt text](https://github.com/f00kat/stm32f3-hc-sr04/blob/master/img/pwm_output.png)

pwm input</br>
![alt text](https://github.com/f00kat/stm32f3-hc-sr04/blob/master/img/pwm_input.png)

schema</br>
![alt text](https://github.com/f00kat/stm32f3-hc-sr04/blob/master/img/schema.png)
