#include <Arduino.h>

#define sensorA0 1
#define sensorD0 41

#define sirenLED 35

#define extinguisher 40


void setup() {
  
  pinMode(sensorD0, INPUT);
  pinMode(sirenLED, OUTPUT);
  pinMode(extinguisher, OUTPUT);
  
  Serial.begin(9600);
}

int counter = 600;

void loop() {

  int firePresence = digitalRead(sensorD0);
  
  
  if(firePresence == HIGH) {

    digitalWrite(sirenLED, HIGH);
    digitalWrite(extinguisher, HIGH);

  } else {

    counter--;

    if(counter == 0) { 

      digitalWrite(sirenLED, LOW);
      digitalWrite(extinguisher, LOW);
      counter = 600;
    }


  }

  int analogue = analogRead(sensorA0);
  Serial.print(analogue);
  Serial.print("    ");
  Serial.print(firePresence);
  Serial.println();

  delay(100);
  // put your main code here, to run repeatedly:
}
