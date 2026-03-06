package com.uiautomator.testapp;

import android.os.Bundle;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.RadioGroup;
import android.widget.Switch;
import android.widget.TextView;
import androidx.appcompat.app.AppCompatActivity;

public class BasicControlsActivity extends AppCompatActivity {

    private TextView tvResult;
    private CheckBox cbOption;
    private RadioGroup rgOptions;
    private Switch swToggle;
    private int clickCount = 0;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_basic_controls);

        tvResult = findViewById(R.id.tv_result);
        cbOption = findViewById(R.id.cb_option);
        rgOptions = findViewById(R.id.rg_options);
        swToggle = findViewById(R.id.sw_toggle);

        setupListeners();
    }

    private void setupListeners() {
        // Normal button
        findViewById(R.id.btn_normal).setOnClickListener(v -> {
            clickCount++;
            tvResult.setText("Button clicked! Count: " + clickCount);
        });

        // Reset button
        findViewById(R.id.btn_reset).setOnClickListener(v -> {
            clickCount = 0;
            cbOption.setChecked(false);
            rgOptions.clearCheck();
            swToggle.setChecked(false);
            tvResult.setText("Result: Reset");
        });

        // CheckBox
        cbOption.setOnCheckedChangeListener((buttonView, isChecked) ->
            tvResult.setText("CheckBox: " + (isChecked ? "Checked" : "Unchecked")));

        // RadioGroup
        rgOptions.setOnCheckedChangeListener((group, checkedId) -> {
            if (checkedId == R.id.rb_option1) {
                tvResult.setText("Radio: Option 1 selected");
            } else if (checkedId == R.id.rb_option2) {
                tvResult.setText("Radio: Option 2 selected");
            } else if (checkedId == R.id.rb_option3) {
                tvResult.setText("Radio: Option 3 selected");
            }
        });

        // Switch
        swToggle.setOnCheckedChangeListener((buttonView, isChecked) ->
            tvResult.setText("Switch: " + (isChecked ? "ON" : "OFF")));
    }
}
