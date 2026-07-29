#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>
#import <Vision/Vision.h>

#include <stdlib.h>
#include <string.h>

static char *dj_copy_string(NSString *value) {
    if (value == nil) {
        return NULL;
    }
    return strdup(value.UTF8String);
}

static NSString *dj_ax_string(AXUIElementRef element, CFStringRef attribute) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, attribute, &value) != kAXErrorSuccess || value == NULL) {
        return @"";
    }
    NSString *result = @"";
    if (CFGetTypeID(value) == CFStringGetTypeID()) {
        result = [NSString stringWithString:(__bridge NSString *)value];
    }
    CFRelease(value);
    return result;
}

static BOOL dj_ax_point(AXUIElementRef element, CFStringRef attribute, CGPoint *point) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, attribute, &value) != kAXErrorSuccess || value == NULL) {
        return NO;
    }
    BOOL ok = NO;
    if (CFGetTypeID(value) == AXValueGetTypeID()) {
        ok = AXValueGetValue((AXValueRef)value, kAXValueCGPointType, point);
    }
    CFRelease(value);
    return ok;
}

static BOOL dj_ax_size(AXUIElementRef element, CFStringRef attribute, CGSize *size) {
    CFTypeRef value = NULL;
    if (AXUIElementCopyAttributeValue(element, attribute, &value) != kAXErrorSuccess || value == NULL) {
        return NO;
    }
    BOOL ok = NO;
    if (CFGetTypeID(value) == AXValueGetTypeID()) {
        ok = AXValueGetValue((AXValueRef)value, kAXValueCGSizeType, size);
    }
    CFRelease(value);
    return ok;
}

int dj_screen_access(void) {
    if (@available(macOS 10.15, *)) {
        return CGPreflightScreenCaptureAccess() ? 1 : 0;
    }
    return 1;
}

int dj_request_screen_access(void) {
    if (@available(macOS 10.15, *)) {
        return CGRequestScreenCaptureAccess() ? 1 : 0;
    }
    return 1;
}

int dj_accessibility_access(void) {
    return AXIsProcessTrusted() ? 1 : 0;
}

int dj_request_accessibility_access(void) {
    NSDictionary *options = @{
        (__bridge NSString *)kAXTrustedCheckOptionPrompt: @YES
    };
    return AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options) ? 1 : 0;
}

char *dj_window_context(char **error_out) {
    @autoreleasepool {
        *error_out = NULL;
        NSRunningApplication *app = NSWorkspace.sharedWorkspace.frontmostApplication;
        if (app == nil) {
            *error_out = dj_copy_string(@"No active application");
            return NULL;
        }

        BOOL trusted = AXIsProcessTrusted();
        NSString *title = @"";
        CGPoint position = CGPointZero;
        CGSize size = CGSizeZero;

        if (trusted) {
            AXUIElementRef appElement = AXUIElementCreateApplication(app.processIdentifier);
            CFTypeRef windowValue = NULL;
            if (AXUIElementCopyAttributeValue(
                    appElement,
                    kAXFocusedWindowAttribute,
                    &windowValue
                ) == kAXErrorSuccess && windowValue != NULL) {
                AXUIElementRef window = (AXUIElementRef)windowValue;
                title = dj_ax_string(window, kAXTitleAttribute);
                dj_ax_point(window, kAXPositionAttribute, &position);
                dj_ax_size(window, kAXSizeAttribute, &size);
                CFRelease(windowValue);
            }
            CFRelease(appElement);
        }

        NSDictionary *result = @{
            @"appName": app.localizedName ?: @"",
            @"bundleId": app.bundleIdentifier ?: @"",
            @"processId": @(app.processIdentifier),
            @"windowTitle": title,
            @"windowX": @(position.x),
            @"windowY": @(position.y),
            @"windowWidth": @(size.width),
            @"windowHeight": @(size.height),
            @"accessibilityGranted": @(trusted)
        };

        NSError *error = nil;
        NSData *data = [NSJSONSerialization dataWithJSONObject:result options:0 error:&error];
        if (data == nil) {
            *error_out = dj_copy_string(error.localizedDescription ?: @"Could not encode window context");
            return NULL;
        }
        NSString *json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
        char *value = dj_copy_string(json);
        [json release];
        return value;
    }
}

char *dj_recognize_image(const char *path, char **error_out) {
    @autoreleasepool {
        *error_out = NULL;
        NSString *pathValue = [NSString stringWithUTF8String:path];
        NSURL *url = [NSURL fileURLWithPath:pathValue];
        VNImageRequestHandler *handler = [[VNImageRequestHandler alloc] initWithURL:url options:@{}];
        VNRecognizeTextRequest *request = [[VNRecognizeTextRequest alloc] init];
        request.recognitionLevel = VNRequestTextRecognitionLevelAccurate;
        request.usesLanguageCorrection = YES;
        request.recognitionLanguages = @[@"en-US", @"zh-Hans", @"zh-Hant"];

        NSError *error = nil;
        BOOL ok = [handler performRequests:@[request] error:&error];
        if (!ok) {
            *error_out = dj_copy_string(error.localizedDescription ?: @"OCR failed");
            [request release];
            [handler release];
            return NULL;
        }

        NSMutableArray *blocks = [NSMutableArray array];
        for (VNRecognizedTextObservation *observation in request.results) {
            VNRecognizedText *candidate = [observation topCandidates:1].firstObject;
            if (candidate == nil || candidate.string.length == 0) {
                continue;
            }
            CGRect box = observation.boundingBox;
            [blocks addObject:@{
                @"text": candidate.string,
                @"confidence": @(candidate.confidence),
                @"x": @(box.origin.x),
                @"y": @(1.0 - CGRectGetMaxY(box)),
                @"width": @(box.size.width),
                @"height": @(box.size.height)
            }];
        }

        NSData *data = [NSJSONSerialization dataWithJSONObject:blocks options:0 error:&error];
        if (data == nil) {
            *error_out = dj_copy_string(error.localizedDescription ?: @"Could not encode OCR result");
            [request release];
            [handler release];
            return NULL;
        }
        NSString *json = [[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];
        char *value = dj_copy_string(json);
        [json release];
        [request release];
        [handler release];
        return value;
    }
}

void dj_free_string(char *value) {
    free(value);
}
