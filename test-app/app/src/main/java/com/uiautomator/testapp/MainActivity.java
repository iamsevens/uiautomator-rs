package com.uiautomator.testapp;

import android.content.Intent;
import android.os.Bundle;
import androidx.appcompat.app.AppCompatActivity;

public class MainActivity extends AppCompatActivity {

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        setupButtons();
    }

    private void setupButtons() {
        bindNavigationButton(R.id.btn_basic_controls, BasicControlsActivity.class);
        bindNavigationButton(R.id.btn_gestures, GesturesActivity.class);
        bindNavigationButton(R.id.btn_input_forms, InputFormsActivity.class);
        bindNavigationButton(R.id.btn_lists, ListsActivity.class);
        bindNavigationButton(R.id.btn_dialogs, DialogsActivity.class);
        bindNavigationButton(R.id.btn_navigation, NavigationActivity.class);
        bindNavigationButton(R.id.btn_animations, AnimationsActivity.class);
        bindNavigationButton(R.id.btn_stress, StressTestActivity.class);
        bindNavigationButton(R.id.btn_concurrent, ConcurrentTestActivity.class);
    }

    private void bindNavigationButton(int buttonId, Class<? extends AppCompatActivity> activityClass) {
        findViewById(buttonId).setOnClickListener(v ->
            startActivity(new Intent(this, activityClass)));
    }
}
