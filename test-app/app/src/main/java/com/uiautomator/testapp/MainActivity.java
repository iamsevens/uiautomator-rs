package com.uiautomator.testapp;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import androidx.appcompat.app.AppCompatActivity;

public class MainActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        setupButtons();
    }

    private void setupButtons() {
        findViewById(R.id.btn_basic_controls).setOnClickListener(v ->
            startActivity(new Intent(this, BasicControlsActivity.class)));

        findViewById(R.id.btn_gestures).setOnClickListener(v ->
            startActivity(new Intent(this, GesturesActivity.class)));

        findViewById(R.id.btn_input_forms).setOnClickListener(v ->
            startActivity(new Intent(this, InputFormsActivity.class)));

        findViewById(R.id.btn_lists).setOnClickListener(v ->
            startActivity(new Intent(this, ListsActivity.class)));

        findViewById(R.id.btn_dialogs).setOnClickListener(v ->
            startActivity(new Intent(this, DialogsActivity.class)));

        findViewById(R.id.btn_navigation).setOnClickListener(v ->
            startActivity(new Intent(this, NavigationActivity.class)));

        findViewById(R.id.btn_animations).setOnClickListener(v ->
            startActivity(new Intent(this, AnimationsActivity.class)));

        findViewById(R.id.btn_stress).setOnClickListener(v ->
            startActivity(new Intent(this, StressTestActivity.class)));

        findViewById(R.id.btn_concurrent).setOnClickListener(v ->
            startActivity(new Intent(this, ConcurrentTestActivity.class)));
    }
}
